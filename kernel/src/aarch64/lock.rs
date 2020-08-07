use super::cpu;
use crate::driver::topology::MAX_CPUS_PER_CLUSTER;

use core::intrinsics::volatile_load;

pub struct LockVar {
    var: u64,
}

/// ```
/// let var = LockVar::new(); // create lock variable
/// var.lock();               // acquire lock
/// ```
impl LockVar {
    pub const fn new() -> LockVar {
        LockVar { var: 0 }
    }

    pub fn lock(&mut self) -> SpinLock {
        SpinLock::new(&mut self.var)
    }
}

pub struct SpinLock<'a> {
    lock: &'a mut u64,
}

impl<'a> SpinLock<'a> {
    fn new(n: &'a mut u64) -> SpinLock<'a> {
        if 0 == unsafe { volatile_load(n) } {
            if test_and_set_no_release(n) {
                return SpinLock { lock: n };
            }
        }

        loop {
            cpu::send_event_local();
            loop {
                cpu::wait_event();
                if 0 == unsafe { volatile_load(n) } {
                    break;
                }
            }

            if test_and_set_no_release(n) {
                return SpinLock { lock: n };
            }
        }
    }
}

impl<'a> Drop for SpinLock<'a> {
    fn drop(&mut self) {
        let addr = self.lock as *mut u64 as usize;
        unsafe {
            asm!("stlr xzr, [{}]", in(reg) addr);
        }
    }
}

/// ```
/// let ticket = BakeryTicket::new(); // create lock variable
/// ticket.lock();                    // acquire lock
/// ```
pub struct BakeryTicket {
    entering: [bool; MAX_CPUS_PER_CLUSTER as usize],
    number: [usize; MAX_CPUS_PER_CLUSTER as usize],
}

impl BakeryTicket {
    pub const fn new() -> BakeryTicket {
        BakeryTicket {
            entering: [false; MAX_CPUS_PER_CLUSTER as usize],
            number: [0; MAX_CPUS_PER_CLUSTER as usize],
        }
    }

    pub fn lock(&mut self) -> BakeryLock {
        BakeryLock::new(self)
    }
}

pub struct BakeryLock<'a> {
    ticket: &'a mut BakeryTicket,
}

impl<'a> BakeryLock<'a> {
    fn new(t: &'a mut BakeryTicket) -> BakeryLock<'a> {
        let core = cpu::get_affinity_lv0() as usize;
        t.entering[core] = true;
        cpu::dmb_sy();
        let mut max = 0;
        for v in &t.number {
            if max < *v {
                max = *v;
            }
        }
        t.number[core] = 1 + max;
        t.entering[core] = false;
        cpu::dmb_sy();

        for i in 0..(MAX_CPUS_PER_CLUSTER as usize) {
            while t.entering[i] {}

            while t.number[i] != 0 && (t.number[i], i) < (t.number[core], core) {}
        }

        BakeryLock { ticket: t }
    }
}

impl<'a> Drop for BakeryLock<'a> {
    fn drop(&mut self) {
        let core = cpu::get_affinity_lv0() as usize;
        self.ticket.number[core] = 0;
    }
}

/// load-acquire and store exclusive
fn test_and_set_no_release(n: &mut u64) -> bool {
    let mut rd: u64;
    let addr = n as *mut u64 as u64;
    unsafe {
        asm! (
            "mov   {2}, #1
             1:
             ldaxr {3}, [{0}]
             stxr  {4:w}, {2}, [{0}]
             cbnz  {4:w}, 1b
             and   {1}, {3}, #1",
            in(reg) addr,
            lateout(reg) rd,
            out(reg) _,
            out(reg) _,
            out(reg) _,
        );
    }
    rd == 0
}

/// load-acquire and store-release exclusive
fn test_and_set(n: &mut u64) -> bool {
    let mut rd: u64;
    let addr = n as *mut u64 as u64;
    unsafe {
        asm! (
            "mov   {2}, #1
             1:
             ldaxr {3}, [{0}]
             stlxr {4:w}, {2}, [{0}]
             cbnz  {4:w}, 1b
             and   {1}, {3}, #1",
            in(reg) addr,
            lateout(reg) rd,
            out(reg) _,
            out(reg) _,
            out(reg) _,
        );
    }
    rd == 0
}
