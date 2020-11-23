use super::cpu;
use crate::driver::topology::{core_pos, CORE_COUNT};

use core::ptr::{read_volatile, write_volatile};

/// ```
/// let var = LockVar::new(); // create lock variable
/// var.lock();               // acquire lock
/// ```
pub struct LockVar {
    var: u64,
}

impl LockVar {
    pub const fn new() -> LockVar {
        LockVar { var: 0 }
    }

    pub fn lock(&mut self) -> SpinLock {
        SpinLock::new(&mut self.var)
    }

    /// this function lock but do not release automatically
    pub unsafe fn force_lock(&mut self) {
        lock_var(&mut self.var);
    }

    /// unlock
    pub unsafe fn force_unlock(&mut self) {
        unlock_var(&mut self.var);
    }
}

pub struct SpinLock<'a> {
    lock: &'a mut u64,
}

impl<'a> SpinLock<'a> {
    fn new(n: &'a mut u64) -> SpinLock<'a> {
        lock_var(n);
        return SpinLock { lock: n };
    }
}

impl<'a> Drop for SpinLock<'a> {
    fn drop(&mut self) {
        unlock_var(&mut self.lock);
    }
}

fn lock_var<'a>(n: &'a mut u64) {
    if 0 == unsafe { read_volatile(n) } {
        if test_and_set_no_release(n) {
            return;
        }
    }

    loop {
        cpu::send_event_local();
        loop {
            cpu::wait_event();
            if 0 == unsafe { read_volatile(n) } {
                break;
            }
        }

        if test_and_set_no_release(n) {
            return;
        }
    }
}

fn unlock_var<'a>(n: &'a mut u64) {
    let addr = n as *mut u64 as usize;
    unsafe {
        asm!("stlr xzr, [{}]", in(reg) addr);
    }
}

/// ```
/// let ticket = BakeryTicket::new(); // create lock variable
/// ticket.lock();                    // acquire lock
/// ```
pub struct BakeryTicket {
    entering: [bool; CORE_COUNT as usize],
    number: [usize; CORE_COUNT as usize],
}

impl BakeryTicket {
    pub const fn new() -> BakeryTicket {
        BakeryTicket {
            entering: [false; CORE_COUNT as usize],
            number: [0; CORE_COUNT as usize],
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
        let core = core_pos() as usize;
        cpu::dmb_sy();
        unsafe {
            write_volatile(&mut t.entering[core], true);
        }
        cpu::dmb_sy();
        let mut max = 0;
        for v in &t.number {
            if max < *v {
                max = *v;
            }
        }
        let ticket = 1 + max;
        unsafe {
            write_volatile(&mut t.number[core], ticket);
            cpu::dmb_sy();
            write_volatile(&mut t.entering[core], false);
            cpu::dmb_sy();
        }

        for i in 0..(CORE_COUNT as usize) {
            if i == core {
                continue;
            }

            cpu::dmb_sy();
            while unsafe { read_volatile(&t.entering[i]) } {}
            cpu::dmb_sy();

            let mut n = unsafe { read_volatile(&t.number[i]) };
            while n != 0 && (n, i) < (ticket, core) {
                n = unsafe { read_volatile(&t.number[i]) };
            }
        }

        cpu::dmb_sy();
        BakeryLock { ticket: t }
    }
}

impl<'a> Drop for BakeryLock<'a> {
    fn drop(&mut self) {
        let core = core_pos() as usize;
        cpu::dmb_sy();
        unsafe { write_volatile(&mut self.ticket.number[core], 0) };
        cpu::dmb_sy();
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
