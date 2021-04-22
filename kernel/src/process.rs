use crate::aarch64::context::GpRegs;
use crate::aarch64::{cpu, mmu};
use crate::driver::topology::{core_pos, CORE_COUNT};

use alloc::alloc;
use core::ptr::null_mut;
use synctools::mcs::{MCSLock, MCSLockGuard, MCSNode};

const PROCESS_MAX: usize = 256;
const STACK_SIZE: usize = 2 * 1024 * 1024;

static mut LOCK: MCSLock<()> = MCSLock::new(());
static mut PROCESS_TABLE: [Process; PROCESS_MAX] = [Process::new(0); PROCESS_MAX];
static mut ACTIVES: [Option<u8>; CORE_COUNT] = [None; CORE_COUNT];
static mut READYQ: ProcessQ = ProcessQ(None);
static mut FREE_STACK: [Option<*mut u8>; CORE_COUNT] = [None; CORE_COUNT];

fn lock<'t>(node: &'t mut MCSNode<()>) -> MCSLockGuard<'t, ()> {
    unsafe { LOCK.lock(node) }
}

fn get_process_table() -> &'static mut [Process; PROCESS_MAX] {
    unsafe { &mut PROCESS_TABLE }
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
    fn enqueue(&mut self, id: u8, tbl: &mut [Process; PROCESS_MAX]) {
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

#[derive(Copy, Clone)]
pub enum State {
    Ready,
    Active,
    Dead,
}

#[derive(Copy, Clone)]
struct Process {
    regs: GpRegs,
    state: State,
    next: Option<u8>,
    stack: *mut u8,
    id: u8,
}

impl Process {
    pub const fn new(id: u8) -> Process {
        Process {
            regs: GpRegs::new(),
            state: State::Dead,
            next: None,
            stack: null_mut(),
            id,
        }
    }
}

struct InterMask {
    prev: u64,
}

impl InterMask {
    fn new() -> InterMask {
        // disable FIQ, IRQ, Abort, Debug
        let prev = cpu::daif::get();
        cpu::daif::set(prev | cpu::DISABLE_ALL_EXCEPTIONS);

        InterMask { prev }
    }

    fn unmask(self) {}
}

impl Drop for InterMask {
    fn drop(&mut self) {
        // restore DAIF
        cpu::daif::set(self.prev);
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

    // enqueue the process to Ready queue
    get_readyq().enqueue(0, tbl);

    schedule2(mask, lock);
}

#[no_mangle]
fn goto_el0() {
    unsafe { asm!("eret") }
}

fn init_process(id: usize) {
    let tbl = get_process_table();

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

    // TODO: set canary
    // TODO: allocate process's heap
}

/// Spawn a new process.
/// If successful this function is unreachable, otherwise (fail) this returns normally.
pub fn spawn(app: u64) -> Option<u8> {
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
    get_readyq().enqueue(id, tbl);
    tbl[id as usize].regs.x0 = app;

    schedule2(mask, lock);

    Some(id)
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
            ready.enqueue(current, tbl);

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
pub fn get_id() -> u8 {
    let aff = core_pos();

    // disable FIQ, IRQ, Abort, Debug
    let _mask = InterMask::new();

    let actives = get_actives();
    actives[aff].unwrap()
}
