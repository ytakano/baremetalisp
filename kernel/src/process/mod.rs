mod int;
mod ringq;
mod semaphore;

use crate::aarch64::context::GpRegs;
use crate::aarch64::mmu;
use crate::driver::topology::{core_pos, CORE_COUNT};

use int::InterMask;

use alloc::alloc;
use core::{
    mem::{size_of, transmute},
    ptr::null_mut,
};
use synctools::mcs::{MCSLock, MCSLockGuard, MCSNode};

const PROCESS_MAX: usize = 256;
const STACK_SIZE: usize = 2 * 1024 * 1024;
const QUEUE_SIZE: usize = 8;

static mut LOCK: MCSLock<()> = MCSLock::new(());
static mut ACTIVES: [Option<u8>; CORE_COUNT] = [None; CORE_COUNT];
static mut READYQ: ProcessQ = ProcessQ(None);
static mut FREE_STACK: [Option<*mut u8>; CORE_COUNT] = [None; CORE_COUNT];

static mut PROCESS_TABLE_BUF: [u8; size_of::<[Process; PROCESS_MAX]>()] =
    [0; size_of::<[Process; PROCESS_MAX]>()];

fn lock<'t>(node: &'t mut MCSNode<()>) -> MCSLockGuard<'t, ()> {
    unsafe { LOCK.lock(node) }
}

fn get_process_table() -> &'static mut [Process; PROCESS_MAX] {
    unsafe {
        let tbl = transmute(&mut PROCESS_TABLE_BUF);
        tbl
    }
}

fn get_actives() -> &'static mut [Option<u8>; CORE_COUNT] {
    unsafe { &mut ACTIVES }
}

fn get_readyq() -> &'static mut ProcessQ {
    unsafe { &mut READYQ }
}

fn get_free_stack() -> &'static mut [Option<*mut u8>; CORE_COUNT] {
    unsafe { &mut FREE_STACK }
}

struct ProcessQ(Option<(u8, u8)>); // (head, tail)

impl ProcessQ {
    fn new() -> ProcessQ {
        ProcessQ(None)
    }

    fn is_empty(&self) -> bool {
        self.0.is_none()
    }

    fn enque(&mut self, id: u8, tbl: &mut [Process; PROCESS_MAX]) {
        match self.0 {
            Some((h, t)) => {
                tbl[t as usize].next = Some(id);
                self.0 = Some((h, id));
            }
            None => {
                self.0 = Some((id, id));
            }
        }
    }

    fn deque(&mut self, tbl: &mut [Process; PROCESS_MAX]) -> Option<u8> {
        match self.0 {
            Some((h, t)) => {
                let hidx = h as usize;
                if let Some(next) = tbl[hidx].next {
                    self.0 = Some((next, t));
                } else {
                    assert_eq!(h, t);
                    self.0 = None;
                }

                tbl[hidx].next = None;
                Some(tbl[hidx].id)
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
    Dead,
}

struct Process {
    regs: GpRegs,
    state: State,
    next: Option<u8>,
    stack: *mut u8,
    tx: Option<ringq::Sender<u32, QUEUE_SIZE>>,
    rx: Option<ringq::Receiver<u32, QUEUE_SIZE>>,
    id: u8,
    cnt: u16,
}

impl Process {
    pub fn new(id: u8) -> Process {
        Process {
            regs: GpRegs::new(),
            state: State::Dead,
            next: None,
            stack: null_mut(),
            tx: None,
            rx: None,
            id: id,
            cnt: 0,
        }
    }

    fn get_pid(&self) -> u32 {
        (self.cnt as u32) << 8 | self.id as u32
    }

    fn pid_to_id_cnt(pid: u32) -> (u8, u16) {
        ((pid & 0xff) as u8, (pid >> 8) as u16)
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

    // aqcuire lock
    let mut node = MCSNode::new();
    let lock = lock(&mut node);

    let tbl = get_process_table();

    // initialize the init process
    init_process(0);

    // enque the process to Ready queue
    get_readyq().enque(0, tbl);

    schedule2(mask, lock);
}

#[no_mangle]
fn goto_el0() {
    unsafe { asm!("eret") }
}

fn init_process(id: usize) {
    let tbl = get_process_table();
    for entry in tbl.iter_mut() {
        entry.state = State::Dead;
    }

    // allocate stack
    let layout = alloc::Layout::from_size_align(STACK_SIZE, mmu::PAGESIZE as usize).unwrap();
    tbl[id].stack = unsafe { alloc::alloc(layout) };

    // initialize
    tbl[id].state = State::Ready;
    tbl[id].next = None;
    tbl[id].id = id as u8;
    tbl[id].regs.spsr = 0; // EL0t
    tbl[id].regs.elr = el0_entry as u64;
    tbl[id].regs.sp = tbl[id].stack as u64;
    tbl[id].regs.x30 = goto_el0 as u64;
    tbl[id].cnt += 1;

    let (tx, rx) = ringq::Chan::<u32, QUEUE_SIZE>::new(id as u8);
    tbl[id].tx = Some(tx);
    tbl[id].rx = Some(rx);

    //let b = Box::new(ringq::Chan::<u32, 8>::new(id as u8));
    //let ptr = Box::into_raw(b);
    //unsafe { Box::from_raw(ptr) };
    //tbl[id].q = ptr;

    // TODO: set canary
    // TODO: allocate process's heap
}

/// Spawn a new process.
/// If successful this function is unreachable, otherwise (fail) this returns normally.
pub fn spawn(app: u64) -> Option<u32> {
    gc_stack(); // garbage collection

    // disable FIQ, IRQ, Abort, Debug
    let mask = InterMask::new();

    // aqcuire lock
    let mut node = MCSNode::new();
    let lock = lock(&mut node);

    let tbl = get_process_table();

    // find empty slot
    let mut id: Option<u8> = None;
    for i in 0..PROCESS_MAX {
        if let State::Dead = tbl[i].state {
            id = Some(i as u8);
            break;
        }
    }

    let id = id?;

    // initialize process
    init_process(id as usize);
    get_readyq().enque(id, tbl);
    tbl[id as usize].regs.x0 = app;

    schedule2(mask, lock);

    Some(tbl[id as usize].get_pid())
}

/// exit process
/// this function is always unreachable
pub fn exit() -> ! {
    gc_stack(); // garbage collection

    // disable FIQ, IRQ, Abort, Debug
    let mask = InterMask::new();

    // aqcuire lock
    let mut node = MCSNode::new();
    let lock = lock(&mut node);

    let tbl = get_process_table();

    let aff = core_pos();
    let actives = get_actives();
    if let Some(current) = actives[aff] {
        tbl[current as usize].state = State::Dead;
        tbl[current as usize].tx = None;
        tbl[current as usize].rx = None;
        get_free_stack()[aff] = Some(tbl[current as usize].stack); // stack to be freed
    }

    actives[aff] = None;

    // TODO: unset canary
    // TODO: deallocate process's heap

    schedule2(mask, lock);
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

fn schedule2(mask: InterMask, lock: MCSLockGuard<()>) {
    // get next
    let tbl = get_process_table();
    let ready = get_readyq();
    let actives = get_actives();
    let next = ready.deque(tbl);
    let aff = core_pos();

    if let Some(next) = next {
        // move the current process to Ready queue
        if let Some(current) = actives[aff] {
            tbl[current as usize].state = State::Ready;
            ready.enque(current, tbl);

            // make the next process Active
            actives[aff] = Some(next);
            tbl[next as usize].state = State::Active;

            lock.unlock();
            mask.unmask();

            if tbl[current as usize].regs.save_context() > 0 {
                return;
            }
        } else {
            // make the next process Active
            actives[aff] = Some(next);
            tbl[next as usize].state = State::Active;

            lock.unlock();
            mask.unmask();
        }

        // context switch
        tbl[next as usize].regs.context_switch();
    } else if None == actives[aff] {
        lock.unlock();
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
    let lock = lock(&mut node);

    schedule2(mask, lock);
}

/// Get the process ID.
pub fn get_id() -> u32 {
    let aff = core_pos();

    // disable FIQ, IRQ, Abort, Debug
    let _mask = InterMask::new();

    let actives = get_actives();
    let id = actives[aff].unwrap();
    get_process_table()[id as usize].get_pid()
}
