use core::intrinsics::volatile_load;

use super::memory::*;

const SYSTMR_LO: *mut u32 = (MMIO_BASE + 0x00003004) as *mut u32;
const SYSTMR_HI: *mut u32 = (MMIO_BASE + 0x00003008) as *mut u32;

// Wait N CPU cycles (ARM CPU only)
pub fn wait_cycles(n: u32) -> () {
    if n > 0 {
        for _ in 0..n {
            unsafe { asm!("nop;") };
        }
    }
}

// Wait N microsec (ARM CPU only)
pub fn wait_microsec(n: u32) -> () {
    // get the current counter frequency
    let mut frq: u64;
    unsafe { asm!("mrs %0, cntfrq_el0" : "=r"(frq)) };

    // read the current counter
    let mut t: u64;
    unsafe { asm!("mrs %0, cntpct_el0" : "=r"(t)) };

    t += ((frq / 1000) * n as u64) / 1000;

    let mut r: u64;
    unsafe { asm!("mrs %0, cntpct_el0" : "=r"(r)) };
    while r < t {
        unsafe { asm!("mrs %0, cntpct_el0" : "=r"(r)) };
    }
}

// Get System Timer's counter
pub fn get_system_timer() -> u64 {
    let mut hi: u32;
    let mut lo: u32;

    unsafe {
        hi = volatile_load(SYSTMR_HI);
        lo = volatile_load(SYSTMR_LO);
    }

    if hi != unsafe { volatile_load(SYSTMR_HI) } {
        unsafe {
            hi = volatile_load(SYSTMR_HI);
            lo = volatile_load(SYSTMR_LO);
        }
    }

    (hi as u64) << 32 | lo as u64
}

// Wait N microsec (with BCM System Timer)
pub fn wait_microsec_st(n: u32) {
    let t = get_system_timer();
    // we must check if it's non-zero, because qemu does not emulate
    // system timer, and returning constant zero would mean infinite loop
    if t > 0 {
        while get_system_timer() < t + n as u64 { }
    }
}