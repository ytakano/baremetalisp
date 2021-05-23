mod ringq;

use crate::{
    aarch64::{context::GpRegs, cpu},
    allocator::{unset_user_allocator, user_stack},
    cpuint::{self, InterMask},
    driver::topology::{core_pos, CORE_COUNT},
    paging,
    syscall::Locator,
};

use arr_macro::arr;
use core::ptr::{null, null_mut};
use synctools::mcs::{MCSLock, MCSLockGuard, MCSNode};

pub const PROCESS_MAX: usize = 256;

static mut ACTIVES: [Option<u8>; CORE_COUNT] = [None; CORE_COUNT];
static mut RECEIVER: [*const ringq::Chan<Msg>; PROCESS_MAX] = [null(); PROCESS_MAX];
static mut FREED: [Option<u8>; CORE_COUNT] = [None; CORE_COUNT];

static PROC_INFO: MCSLock<ProcInfo> = MCSLock::new(ProcInfo::new());

#[derive(Clone)]
struct Msg {
    loc: Locator,
    val: u32,
}

unsafe impl Send for Msg {}

struct ProcInfo {
    table: [Option<Process>; PROCESS_MAX],
    cnt: [u16; PROCESS_MAX],
    readyq: ProcessQ,
}

impl ProcInfo {
    const fn new() -> ProcInfo {
        ProcInfo {
            table: arr![None; 256], // PROCESS_MAX == 256
            cnt: arr![0; 256],
            readyq: ProcessQ::new(),
        }
    }

    fn split(
        &mut self,
    ) -> (
        &mut [Option<Process>; PROCESS_MAX],
        &mut [u16; PROCESS_MAX],
        &mut ProcessQ,
    ) {
        (&mut self.table, &mut self.cnt, &mut self.readyq)
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

fn get_freed() -> &'static mut [Option<u8>; CORE_COUNT] {
    unsafe { &mut FREED }
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
                    self.0 = Some((h, id));
                }
            }
            None => {
                self.0 = Some((id, id));
            }
        }
    }

    fn remove(&mut self, id: u8, tbl: &mut [Option<Process>; PROCESS_MAX]) {
        if let Some((h, t)) = self.0 {
            if h == id {
                if let Some(entry) = tbl[h as usize].as_mut() {
                    if h == t {
                        self.0 = None
                    } else {
                        self.0 = Some((entry.next.unwrap(), t));
                    }
                    entry.next = None;
                    return;
                }
            }

            let id_next = tbl[id as usize].as_ref().unwrap().next;
            let mut n = h;
            loop {
                if let Some(entry) = tbl[n as usize].as_mut() {
                    if let Some(next) = entry.next {
                        if next == id {
                            entry.next = id_next;
                            if t == id {
                                self.0 = Some((h, entry.id));
                            }
                            break;
                        } else {
                            n = next;
                        }
                    } else {
                        return; // not found
                    }
                }
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
    Killed,
    Zombie,
}

struct Process {
    regs: GpRegs,
    state: State,
    next: Option<u8>,
    stack: *mut u8,
    tx: *const ringq::Chan<Msg>,
    id: u8,
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
        }
    }

    fn get_pid(&self, cnt: u16) -> u32 {
        (cnt as u32) << 8 | self.id as u32
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
    fn userland_entry();
}

/// Initialize the first process.
/// It is often called the init process.
pub fn init() {
    // disable FIQ, IRQ, Abort, Debug
    let mask = cpuint::mask();

    let ch = ringq::Chan::<Msg>::new(0);
    let (tx, rx) = ch.channel();

    // allocate stack
    let stack = user_stack(0);

    // let tbl = get_process_table();
    let mut node = MCSNode::new();
    let mut proc_info = PROC_INFO.lock(&mut node);

    // initialize the init process
    init_process(0, &mut proc_info, stack, tx.into_raw(), rx.into_raw());

    // enque the process to Ready queue
    let (tbl, _, readyq) = proc_info.split();
    readyq.enque(0, tbl);

    schedule2(mask, proc_info);
}

#[no_mangle]
fn goto_userland(_app: usize, next: usize, cnt: usize) {
    set_tpid_reg(next as u8, cnt as u16);
    unsafe { asm!("eret") }
}

fn init_process(
    id: usize,
    proc_info: &mut MCSLockGuard<ProcInfo>,
    stack: *mut u8,
    tx: *const ringq::Chan<Msg>,
    rx: *const ringq::Chan<Msg>,
) {
    // initialize
    let mut proc = Process::new(0);
    proc.state = State::Ready;
    proc.next = None;
    proc.id = id as u8;
    proc.regs.spsr = 0; // EL0t
    proc.regs.elr = userland_entry as u64;
    proc.regs.sp = stack as u64;
    proc.regs.x30 = goto_userland as u64;
    proc.stack = stack;
    proc.tx = tx;

    unsafe { RECEIVER[id] = rx };

    proc_info.cnt[id] += 1;
    proc_info.table[id] = Some(proc);
}

/// Spawn a new process.
/// If successful this function is unreachable, otherwise (fail) this returns normally.
pub fn spawn(app: u64) -> Option<u32> {
    // disable FIQ, IRQ, Abort, Debug
    let mask = cpuint::mask();

    // create channel
    let mut ch = ringq::Chan::<Msg>::new(0);

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

    let stack = user_stack(id);

    // initialize process
    init_process(
        id as usize,
        &mut proc_info,
        stack,
        tx.into_raw(),
        rx.into_raw(),
    );

    let (tbl, cnt, readyq) = proc_info.split();
    readyq.enque(id, tbl);
    let pid = {
        if let Some(entry) = tbl[id as usize].as_mut() {
            let cnt2 = cnt[id as usize];
            entry.regs.x0 = app;
            entry.regs.x1 = id as u64;
            entry.regs.x2 = cnt2 as u64;
            entry.get_pid(cnt2)
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
    // disable FIQ, IRQ, Abort, Debug
    let mask = cpuint::mask();

    // aqcuire lock
    let mut node = MCSNode::new();
    let mut proc_info = PROC_INFO.lock(&mut node);

    let tbl = &mut proc_info.table;

    let aff = core_pos();
    let actives = get_actives();
    if let Some(current) = actives[aff] {
        if let Some(entry) = tbl[current as usize].as_mut() {
            unsafe {
                if entry.tx.is_null() {
                    ringq::Sender::from_raw(entry.tx);
                }

                let rx = RECEIVER[current as usize];
                if rx.is_null() {
                    ringq::Receiver::from_raw(rx);
                }
            }
            entry.state = State::Zombie;
        }

        unset_user_allocator(current);

        let freed = get_freed();
        freed[aff] = Some(current);
    }

    actives[aff] = None;

    schedule2(mask, proc_info);
    unreachable!()
}

fn schedule2(mask: cpuint::ArchIntMask, mut proc_info: MCSLockGuard<ProcInfo>) {
    // get next
    let (tbl, _, readyq) = proc_info.split();
    let actives = get_actives();
    let next = readyq.deque(tbl);
    let aff = core_pos();

    if let Some(next) = next {
        let next_ctx;

        // move the current process to Ready queue
        if let Some(current) = actives[aff] {
            if let Some(entry) = tbl[current as usize].as_mut() {
                match entry.state {
                    State::Active => {
                        entry.state = State::Ready;
                        readyq.enque(current, tbl);
                    }
                    State::Killed => {
                        proc_info.unlock();
                        mask.unmask();
                        exit();
                    }
                    _ => (),
                }
            } else {
                return;
            }

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
                    let aff = core_pos();
                    let freed = get_freed();
                    if let Some(id) = freed[aff] {
                        // unmap exited process's memory
                        paging::unmap_user_all(id);
                        freed[aff] = None;

                        // disable FIQ, IRQ, Abort, Debug
                        let _mask = cpuint::mask();

                        // clear entry
                        let mut node = MCSNode::new();
                        let mut proc_info = PROC_INFO.lock(&mut node);
                        proc_info.table[id as usize] = None;
                    }

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
    let mask = cpuint::mask();

    // aqcuire lock
    let mut node = MCSNode::new();
    let proc_info = PROC_INFO.lock(&mut node);

    schedule2(mask, proc_info);
}

/// Get the process ID.
pub fn get_pid() -> u32 {
    let aff = core_pos();

    // disable FIQ, IRQ, Abort, Debug
    let _mask = cpuint::mask();

    let actives = get_actives();
    let id = actives[aff].unwrap();

    let mut node = MCSNode::new();
    let proc_info = PROC_INFO.lock(&mut node);
    if let Some(entry) = proc_info.table[id as usize].as_ref() {
        entry.get_pid(proc_info.cnt[id as usize])
    } else {
        0
    }
}

/// Get the raw Process ID.
pub fn get_raw_id() -> Option<u8> {
    let aff = core_pos();

    let actives = get_actives();
    actives[aff]
}

pub fn send(dst: &Locator, val: u32) -> bool {
    let addr = if let Locator::Process(a) = dst {
        a
    } else {
        return false;
    };

    let id = addr & 0xff;
    let count = addr >> 8;

    // disable FIQ, IRQ, Abort, Debug
    let mask = cpuint::mask();
    let mut node = MCSNode::new();
    let mut proc_info = PROC_INFO.lock(&mut node);

    let (tbl, cnt, _) = proc_info.split();

    if let Some(p) = tbl[id as usize].as_mut() {
        if cnt[id as usize] != count as u16 || p.state == State::Zombie {
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

pub struct EnterKernel {
    tpid: u64,
}

impl EnterKernel {
    pub fn new() -> Self {
        let tpid = cpu::tpidr_el0::get();
        cpu::tpidrro_el0::set(1 << 63);
        EnterKernel { tpid }
    }
}

impl Drop for EnterKernel {
    fn drop(&mut self) {
        cpu::tpidr_el0::set(self.tpid);
    }
}

const TPID_KERNEL_FLAG: u64 = 1 << 63;

/// tpidrro_el0 register format
/// | bits    | mean         |
/// |---------|--------------|
/// |  0 -  7 | raw ID       |
/// |  8 - 23 | count        |
/// | 24 - 31 | CPU affinity |
///
/// tpidrro_el0 must be 1 << 63 in kernel space
fn set_tpid_reg(id: u8, cnt: u16) {
    let aff = core_pos() as u64 & 0xff;
    let cnt = cnt as u64;
    let tpid = (aff << 24) | (cnt << 8) | (id as u64);
    cpu::tpidr_el0::set(tpid);
}

/// Make tpidr_el0 kernel space
pub fn set_tpid_kernel() {
    cpu::tpidr_el0::set(TPID_KERNEL_FLAG);
}

/// Get raw ID from tpidrro_el0
/// This function is for EL0
pub fn get_raw_id_user() -> u8 {
    cpu::tpidr_el0::get() as u8
}

/// Get CPU affinity from tpidrro_el0
/// This function is for EL0
pub fn get_affinity_user() -> u8 {
    (cpu::tpidr_el0::get() >> 24) as u8
}

/// Kernel space or user space?
/// This function is for EL0
pub fn is_kernel() -> bool {
    cpu::tpidr_el0::get() & (TPID_KERNEL_FLAG) != 0
}

pub fn kill(pid: u32) {
    // disable FIQ, IRQ, Abort, Debug
    let mask = cpuint::mask();
    let mut node = MCSNode::new();
    let mut proc_info = PROC_INFO.lock(&mut node);

    let (tbl, cnt, q) = proc_info.split();

    // Suicide
    let aff = core_pos();
    let actives = get_actives();
    if let Some(current) = actives[aff] {
        if let Some(p) = tbl[current as usize].as_mut() {
            if pid == p.get_pid(cnt[current as usize]) {
                proc_info.unlock();
                mask.unmask();
                exit();
            }
        }
    }

    let id = pid & 0xff;
    if let Some(p) = tbl[id as usize].as_mut() {
        if pid == p.get_pid(cnt[id as usize]) {
            match p.state {
                State::Ready => {
                    q.remove(id as u8, tbl);
                    kill_proc(id as u8, mask, proc_info);
                }
                State::Recv => {
                    kill_proc(id as u8, mask, proc_info);
                }
                State::Active => {
                    p.state = State::Killed;
                    // TODO
                }
                _ => (),
            }
        }
    }
}

fn kill_proc(id: u8, mask: cpuint::ArchIntMask, mut proc_info: MCSLockGuard<ProcInfo>) {
    let tbl = proc_info.table.as_mut();
    if let Some(entry) = tbl[id as usize].as_mut() {
        unsafe {
            if entry.tx.is_null() {
                ringq::Sender::from_raw(entry.tx);
            }

            let rx = RECEIVER[id as usize];
            if rx.is_null() {
                ringq::Receiver::from_raw(rx);
            }
        }
    }

    tbl[id as usize] = None;

    proc_info.unlock();
    mask.unmask();

    unset_user_allocator(id);

    // unmap killed process's memory
    paging::unmap_user_all(id);
}
