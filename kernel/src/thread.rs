use crate::aarch64::context::GpRegs;
use crate::aarch64::mmu;
use crate::driver::topology::CORE_COUNT;

use alloc::alloc;
use core::ptr::null_mut;
use synctools::mcs::MCSLock;

const THREAD_MAX: usize = 256;
const STACK_SIZE: usize = 2 * 1024 * 1024;

static mut THREAD_TABLE: MCSLock<[Thread; THREAD_MAX]> = MCSLock::new([Thread::new(0); THREAD_MAX]);
static mut ACTIVES: [Option<u8>; CORE_COUNT] = [None; CORE_COUNT];
static mut READY: MCSLock<ThreadQ> = MCSLock::new(ThreadQ(None));

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

extern "C" {
    fn el0_entry_core_0();
    fn el0_entry_core_x();
}

pub fn spawn() -> Option<u8> {
    let mut tbl = unsafe { THREAD_TABLE.lock() };

    // find empty slot
    let mut id: Option<u8> = None;
    for i in 0..THREAD_MAX {
        if let State::Dead = tbl[i].state {
            id = Some(i as u8);
            break;
        }
    }

    // too many thread
    if id == None {
        return None;
    }

    let id0 = id.unwrap();
    let idu = id0 as usize;

    // allocate stack
    let layout = alloc::Layout::from_size_align(STACK_SIZE, mmu::PAGESIZE as usize).unwrap();
    tbl[idu].stack = unsafe { alloc::alloc(layout) };

    // initialize
    tbl[idu].state = State::Ready;
    tbl[idu].next = None;
    tbl[idu].id = id0;
    tbl[idu].regs.spsr = 0; // EL0t
    tbl[idu].regs.elr = el0_entry_core_0 as *const () as u64;

    // TODO: set canary

    // enqueue this to Ready queue
    let mut ready = unsafe { READY.lock() };
    ready.enqueue(id0, &mut tbl);

    id
}
