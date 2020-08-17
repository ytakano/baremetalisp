use core::ptr::read_volatile;

use super::memory::*;

const SYSTMR_LO: *mut u32 = (MMIO_BASE + 0x00003004) as *mut u32;
const SYSTMR_HI: *mut u32 = (MMIO_BASE + 0x00003008) as *mut u32;

/// Get System Timer's counter
pub fn get_timer_value() -> u64 {
    let mut hi: u32;
    let mut lo: u32;

    unsafe {
        hi = read_volatile(SYSTMR_HI);
        lo = read_volatile(SYSTMR_LO);
    }

    if hi != unsafe { read_volatile(SYSTMR_HI) } {
        unsafe {
            hi = read_volatile(SYSTMR_HI);
            lo = read_volatile(SYSTMR_LO);
        }
    }

    (hi as u64) << 32 | lo as u64
}

/// Wait N microsec (with BCM System Timer)
pub fn wait_microsec(n: u32) {
    let t = get_timer_value();
    // we must check if it's non-zero, because qemu does not emulate
    // system timer, and returning constant zero would mean infinite loop
    if t > 0 {
        while get_timer_value() < t + n as u64 {}
    }
}

pub fn init() {}
