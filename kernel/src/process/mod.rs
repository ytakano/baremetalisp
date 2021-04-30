mod int;
mod ringq;
//mod semaphore;

use crate::aarch64::mmu;
use crate::driver::topology::{core_pos, CORE_COUNT};
use crate::{aarch64::context::GpRegs, syscall::Locator};

use int::InterMask;

use ::alloc::alloc;
use arr_macro::arr;
use core::ptr::{null, null_mut};
use synctools::mcs::{MCSLock, MCSLockGuard, MCSNode};

const PROCESS_MAX: usize = 256;
const STACK_SIZE: usize = 2 * 1024 * 1024;

static mut ACTIVES: [Option<u8>; CORE_COUNT] = [None; CORE_COUNT];
static mut FREE_STACK: [Option<*mut u8>; CORE_COUNT] = [None; CORE_COUNT];
static mut RECEIVER: [*const ringq::Chan<Msg>; PROCESS_MAX] = [null(); PROCESS_MAX];

static PROC_INFO: MCSLock<ProcInfo> = MCSLock::new(ProcInfo::new());

#[derive(Clone)]
struct Msg {
    loc: Locator,
    val: u32,
}

unsafe impl Send for Msg {}

struct ProcInfo {
    table: [Option<Process>; PROCESS_MAX],
    readyq: ProcessQ,
}

impl ProcInfo {
    const fn new() -> ProcInfo {
        ProcInfo {
            table: arr![None; 256], // PROCESS_MAX == 256
            readyq: ProcessQ::new(),
        }
    }

    fn get_mut(&mut self) -> (&mut [Option<Process>; PROCESS_MAX], &mut ProcessQ) {
        (&mut self.table, &mut self.readyq)
    }

    unsafe fn get_ctx(&self, i: usize) -> *mut GpRegs {
        if let Some(entry) = self.table[i].as_ref() {
            &(entry.regs) as *const GpRegs as *mut GpRegs
        } else {
            panic!("no such process");
        }
    }
}

fn get_actives() -> &'static mut [Option<u8>; CORE_COUNT] {
    unsafe { &mut ACTIVES }
}

fn get_free_stack() -> &'static mut [Option<*mut u8>; CORE_COUNT] {
    unsafe { &mut FREE_STACK }
}

struct ProcessQ(Option<(u8, u8)>); // (head, tail)

impl ProcessQ {
    const fn new() -> ProcessQ {
        ProcessQ(None)
    }

    fn is_empty(&self) -> bool {
        self.0.is_none()
    }

    fn enque(&mut self, id: u8, tbl: &mut [Option<Process>; PROCESS_MAX]) {
        match self.0 {
            Some((h, t)) => {
                if let Some(entry) = tbl[t as usize].as_mut() {
                    entry.next = Some(id);
                } else {
                    return;
                }
                self.0 = Some((h, id));
            }
            None => {
                self.0 = Some((id, id));
            }
        }
    }

    fn deque(&mut self, tbl: &mut [Option<Process>; PROCESS_MAX]) -> Option<u8> {
        match self.0 {
            Some((h, t)) => {
                let hidx = h as usize;
                if let Some(entry) = tbl[hidx].as_mut() {
                    if let Some(next) = entry.next {
                        self.0 = Some((next, t));
                    } else {
                        assert_eq!(h, t);
                        self.0 = None;
                    }

                    entry.next = None;
                    Some(entry.id)
                } else {
                    None
                }
            }
            None => None,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum State {
    Ready,
    Active,
    Recv,
    SemWait,
}

struct Process {
    regs: GpRegs,
    state: State,
    next: Option<u8>,
    stack: *mut u8,
    tx: *const ringq::Chan<Msg>,
    id: u8,
    cnt: u16,
}

impl Process {
    pub fn new(id: u8) -> Process {
        Process {
            regs: GpRegs::new(),
            state: State::Ready,
            next: None,
            stack: null_mut(),
            tx: null(),
            id,
            cnt: 0,
        }
    }

    fn get_pid(&self) -> u32 {
        (self.cnt as u32) << 8 | self.id as u32
    }

    fn pid_to_id_cnt(pid: u32) -> (u8, u16) {
        ((pid & 0xff) as u8, (pid >> 8) as u16)
    }

    fn get_tx(&mut self) -> ringq::Sender<Msg> {
        assert_ne!(self.tx, null_mut());
        let tx = unsafe { ringq::Sender::from_raw(self.tx) };
        let ret = tx.clone();
        self.tx = tx.into_raw();
        ret
    }
}

extern "C" {
    fn el0_entry();
}

/// Initialize the first process.
/// It is often called the init process.
pub fn init() {
    // disable FIQ, IRQ, Abort, Debug
    let mask = InterMask::new();

    let ch = ringq::Chan::<Msg>::new(0);
    let (tx, rx) = ch.channel();

    // allocate stack
    let layout = alloc::Layout::from_size_align(STACK_SIZE, mmu::PAGESIZE as usize).unwrap();
    let stack = unsafe { alloc::alloc(layout) };

    // let tbl = get_process_table();
    let mut node = MCSNode::new();
    let mut proc_info = PROC_INFO.lock(&mut node);

    // initialize the init process
    init_process(0, &mut proc_info, stack, tx.into_raw(), rx.into_raw());

    // enque the process to Ready queue
    let (tbl, readyq) = proc_info.get_mut();
    readyq.enque(0, tbl);

    schedule2(mask, proc_info);
}

#[no_mangle]
fn goto_el0() {
    unsafe { asm!("eret") }
}

fn init_process(
    id: usize,
    proc_info: &mut MCSLockGuard<ProcInfo>,
    stack: *mut u8,
    tx: *const ringq::Chan<Msg>,
    rx: *const ringq::Chan<Msg>,
) {
    let tbl = &mut proc_info.table;

    // initialize
    let mut proc = Process::new(0);
    proc.state = State::Ready;
    proc.next = None;
    proc.id = id as u8;
    proc.regs.spsr = 0; // EL0t
    proc.regs.elr = el0_entry as u64;
    proc.regs.sp = stack as u64;
    proc.regs.x30 = goto_el0 as u64;
    proc.cnt += 1;
    proc.stack = stack;
    proc.tx = tx;

    unsafe { RECEIVER[id] = rx };

    tbl[id] = Some(proc);

    // TODO: set canary
    // TODO: allocate process's heap
}

/// Spawn a new process.
/// If successful this function is unreachable, otherwise (fail) this returns normally.
pub fn spawn(app: u64) -> Option<u32> {
    gc_stack(); // garbage collection

    // disable FIQ, IRQ, Abort, Debug
    let mask = InterMask::new();

    // create channel
    let mut ch = ringq::Chan::<Msg>::new(0);

    // allocate stack
    let layout = alloc::Layout::from_size_align(STACK_SIZE, mmu::PAGESIZE as usize).unwrap();
    let stack = unsafe { alloc::alloc(layout) };

    // aqcuire lock
    let mut node = MCSNode::new();
    let mut proc_info = PROC_INFO.lock(&mut node);

    let tbl = &proc_info.table;

    // find empty slot
    let mut id: Option<u8> = None;
    //    for i in 0..PROCESS_MAX {
    for (i, item) in tbl.iter().enumerate().take(PROCESS_MAX) {
        if item.is_none() {
            id = Some(i as u8);
            break;
        }
    }

    let id = id?;
    ch.set_pid(id);
    let (tx, rx) = ch.channel();

    // initialize process
    init_process(
        id as usize,
        &mut proc_info,
        stack,
        tx.into_raw(),
        rx.into_raw(),
    );

    let (tbl, readyq) = proc_info.get_mut();
    readyq.enque(id, tbl);
    let pid = {
        if let Some(entry) = proc_info.table[id as usize].as_mut() {
            entry.regs.x0 = app;
            entry.get_pid()
        } else {
            return None;
        }
    };

    schedule2(mask, proc_info);

    Some(pid)
}

/// exit process
/// this function is always unreachable
pub fn exit() -> ! {
    gc_stack(); // garbage collection

    // disable FIQ, IRQ, Abort, Debug
    let mask = InterMask::new();

    // aqcuire lock
    let mut node = MCSNode::new();
    let mut proc_info = PROC_INFO.lock(&mut node);

    let tbl = &mut proc_info.table;

    let aff = core_pos();
    let actives = get_actives();
    if let Some(current) = actives[aff] {
        if let Some(entry) = tbl[current as usize].as_mut() {
            unsafe {
                if entry.tx != null_mut() {
                    ringq::Sender::from_raw(entry.tx);
                }

                let rx = RECEIVER[current as usize];
                if rx != null() {
                    ringq::Receiver::from_raw(rx);
                }
            }
            get_free_stack()[aff] = Some(entry.stack); // stack to be freed
        }
        tbl[current as usize] = None;
    }

    actives[aff] = None;

    // TODO: unset canary
    // TODO: deallocate process's heap

    schedule2(mask, proc_info);
    unreachable!()
}

fn gc_stack() {
    let aff = core_pos();
    if let Some(stack) = get_free_stack()[aff] {
        let layout = alloc::Layout::from_size_align(STACK_SIZE, mmu::PAGESIZE as usize).unwrap();
        unsafe { alloc::dealloc(stack, layout) };
        get_free_stack()[aff] = None;
    }
}

fn schedule2(mask: InterMask, mut proc_info: MCSLockGuard<ProcInfo>) {
    // get next
    let (tbl, readyq) = proc_info.get_mut();
    let actives = get_actives();
    let next = readyq.deque(tbl);
    let aff = core_pos();

    if let Some(next) = next {
        let next_ctx;

        // move the current process to Ready queue
        if let Some(current) = actives[aff] {
            if let Some(entry) = tbl[current as usize].as_mut() {
                entry.state = State::Ready;
            } else {
                return;
            }

            readyq.enque(current, tbl);

            // make the next process Active
            actives[aff] = Some(next);
            if let Some(entry) = tbl[next as usize].as_mut() {
                entry.state = State::Active;
            } else {
                return;
            }

            next_ctx = unsafe { proc_info.get_ctx(next as usize) };
            let current_ctx = unsafe { proc_info.get_ctx(current as usize) };

            proc_info.unlock();
            mask.unmask();

            unsafe {
                if (*current_ctx).save_context() > 0 {
                    return;
                }
            }
        } else {
            // make the next process Active
            actives[aff] = Some(next);
            if let Some(entry) = tbl[next as usize].as_mut() {
                entry.state = State::Active;
            } else {
                return;
            }
            next_ctx = unsafe { proc_info.get_ctx(next as usize) };

            proc_info.unlock();
            mask.unmask();
        }

        // context switch
        unsafe {
            (*next_ctx).context_switch();
        }
    } else if None == actives[aff] {
        proc_info.unlock();
        mask.unmask();

        crate::aarch64::smc::done();
    }
}

/// Yielding.
pub fn schedule() {
    // disable FIQ, IRQ, Abort, Debug
    let mask = InterMask::new();

    // aqcuire lock
    let mut node = MCSNode::new();
    let proc_info = PROC_INFO.lock(&mut node);

    schedule2(mask, proc_info);
}

/// Get the process ID.
pub fn get_id() -> u32 {
    let aff = core_pos();

    // disable FIQ, IRQ, Abort, Debug
    let _mask = InterMask::new();

    let actives = get_actives();
    let id = actives[aff].unwrap();

    let mut node = MCSNode::new();
    let proc_info = PROC_INFO.lock(&mut node);
    if let Some(entry) = proc_info.table[id as usize].as_ref() {
        entry.get_pid()
    } else {
        0
    }
}

pub fn send(dst: &Locator, val: u32) -> bool {
    let addr = if let Locator::Process(a) = dst {
        a
    } else {
        return false;
    };

    let id = addr & 0xff;
    let cnt = addr >> 8;

    // disable FIQ, IRQ, Abort, Debug
    let mask = InterMask::new();
    let mut node = MCSNode::new();
    let mut proc_info = PROC_INFO.lock(&mut node);

    if let Some(p) = proc_info.table[id as usize].as_mut() {
        if p.cnt != cnt as u16 {
            return false;
        }

        let tx = p.get_tx();

        proc_info.unlock();
        mask.unmask();

        tx.send(Msg { loc: *dst, val }).is_ok()
    } else {
        false
    }
}

pub fn recv(src: &mut Locator) -> u32 {
    let aff = core_pos();
    let id = if let Some(id) = get_actives()[aff] {
        id
    } else {
        panic!("no active process");
    };

    let rx = unsafe {
        let ptr = RECEIVER[id as usize];
        assert_ne!(ptr, null());
        RECEIVER[id as usize] = null();
        ringq::Receiver::from_raw(ptr)
    };

    let ret = rx.recv();

    unsafe { RECEIVER[id as usize] = rx.into_raw() };

    *src = ret.loc;
    ret.val
}
