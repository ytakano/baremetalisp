use crate::aarch64::context::GpRegs;
use crate::aarch64::{cpu, mmu};
use crate::driver::topology::{core_pos, CORE_COUNT};

use alloc::alloc;
use core::ptr::null_mut;
use synctools::mcs::{MCSLock, MCSLockGuard, MCSNode};

const THREAD_MAX: usize = 256;
const STACK_SIZE: usize = 2 * 1024 * 1024;

static mut LOCK: MCSLock<()> = MCSLock::new(());
static mut THREAD_TABLE: [Thread; THREAD_MAX] = [Thread::new(0); THREAD_MAX];
static mut ACTIVES: [Option<u8>; CORE_COUNT] = [None; CORE_COUNT];
static mut READY: ThreadQ = ThreadQ(None);
static mut FREE_STACK: [Option<*mut u8>; CORE_COUNT] = [None; CORE_COUNT];

fn lock<'t>(node: &'t mut MCSNode<()>) -> MCSLockGuard<'t, ()> {
    unsafe { LOCK.lock(node) }
}

fn get_thread_table() -> &'static mut [Thread; THREAD_MAX] {
    unsafe { &mut THREAD_TABLE }
}

fn get_actives() -> &'static mut [Option<u8>; CORE_COUNT] {
    unsafe { &mut ACTIVES }
}

fn get_readyq() -> &'static mut ThreadQ {
    unsafe { &mut READY }
}

fn get_free_stack() -> &'static mut [Option<*mut u8>; CORE_COUNT] {
    unsafe { &mut FREE_STACK }
}

struct ThreadQ(Option<(u8, u8)>);

impl ThreadQ {
    fn enqueue(&mut self, id: u8, tbl: &mut [Thread; THREAD_MAX]) {
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

    fn deque(&mut self, tbl: &mut [Thread; THREAD_MAX]) -> Option<u8> {
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
struct Thread {
    regs: GpRegs,
    state: State,
    next: Option<u8>,
    stack: *mut u8,
    id: u8,
}

impl Thread {
    pub const fn new(id: u8) -> Thread {
        Thread {
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

pub fn init() {
    // disable FIQ, IRQ, Abort, Debug
    let mask = InterMask::new();

    // aqcuire lock
    let mut node = MCSNode::new();
    let lock = lock(&mut node);

    let tbl = get_thread_table();

    // initialize init thread
    init_thread(0);

    // enqueue the thread to Ready queue
    get_readyq().enqueue(0, tbl);

    schedule2(mask, lock);
}

fn init_thread(id: usize) {
    let tbl = get_thread_table();

    // allocate stack
    let layout = alloc::Layout::from_size_align(STACK_SIZE, mmu::PAGESIZE as usize).unwrap();
    tbl[id].stack = unsafe { alloc::alloc(layout) };

    // initialize
    tbl[id].state = State::Ready;
    tbl[id].next = None;
    tbl[id].id = 0;
    tbl[id].regs.spsr = 0; // EL0t
    tbl[id].regs.elr = el0_entry as *const () as u64;
    tbl[id].regs.sp = tbl[id].stack as u64;

    // TODO: set canary
}

pub fn spawn(app: u64, regs: Option<&GpRegs>) -> Option<u8> {
    gc_stack(); // garbage collection

    // disable FIQ, IRQ, Abort, Debug
    let mask = InterMask::new();

    // aqcuire lock
    let mut node = MCSNode::new();
    let lock = lock(&mut node);

    let tbl = get_thread_table();

    // find empty slot
    let mut id: Option<u8> = None;
    for i in 0..THREAD_MAX {
        if let State::Dead = tbl[i].state {
            id = Some(i as u8);
            break;
        }
    }

    let id = id?;

    // initialize thread
    init_thread(id as usize);
    get_readyq().enqueue(id, tbl);
    tbl[id as usize].regs.x0 = app;

    // save current thread's context
    let aff = core_pos();
    match (get_actives()[aff], regs) {
        (Some(idx), Some(r)) => {
            let i = idx as usize;
            tbl[i].regs = *r;
            tbl[i].regs.x0 = id as u64; // set return value
        }
        (None, None) => (),
        _ => panic!("invalid state"),
    }

    schedule2(mask, lock);
    unreachable!()
}

pub fn exit() {
    gc_stack(); // garbage collection

    // disable FIQ, IRQ, Abort, Debug
    let mask = InterMask::new();

    // aqcuire lock
    let mut node = MCSNode::new();
    let lock = lock(&mut node);

    let tbl = get_thread_table();

    let aff = core_pos();
    let actives = get_actives();
    if let Some(current) = actives[aff] {
        tbl[current as usize].state = State::Dead;
        get_free_stack()[aff] = Some(tbl[current as usize].stack); // stack to be freed
    }

    actives[aff] = None;

    schedule2(mask, lock);
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
    let tbl = get_thread_table();
    let ready = get_readyq();
    let next = ready.deque(tbl);
    let aff = core_pos();

    if let Some(next) = next {
        // move the current thread to Ready queue
        let actives = get_actives();
        if let Some(current) = actives[aff] {
            tbl[current as usize].state = State::Ready;
            ready.enqueue(current, tbl);
        }

        // make the next thread Active
        actives[aff] = Some(next);
        tbl[next as usize].state = State::Active;

        lock.unlock();
        mask.unmask();

        // context switch
        tbl[next as usize].regs.context_switch();
    } else {
        lock.unlock();
        mask.unmask();

        crate::aarch64::smc::done();
    }
}

pub fn schedule() {
    // disable FIQ and IRQ
    let mask = InterMask::new();

    // aqcuire lock
    let mut node = MCSNode::new();
    let lock = lock(&mut node);

    schedule2(mask, lock);
}
