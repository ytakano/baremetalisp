use core::intrinsics::volatile_load;

use super::memory::*;

const SYSTMR_LO: *mut u32 = (MMIO_BASE + 0x00003004) as *mut u32;
const SYSTMR_HI: *mut u32 = (MMIO_BASE + 0x00003008) as *mut u32;

/// Get System Timer's counter
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

/// Wait N microsec (with BCM System Timer)
pub fn wait_microsec_st(n: u32) {
    let t = get_system_timer();
    // we must check if it's non-zero, because qemu does not emulate
    // system timer, and returning constant zero would mean infinite loop
    if t > 0 {
        while get_system_timer() < t + n as u64 { }
    }
}