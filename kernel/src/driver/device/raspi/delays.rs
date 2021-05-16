use super::memory::*;
use crate::driver::delays;
use core::ptr::read_volatile;

const SYSTMR_LO: *mut u32 = (MMIO_BASE + 0x00003004) as *mut u32;
const SYSTMR_HI: *mut u32 = (MMIO_BASE + 0x00003008) as *mut u32;

pub(in crate::driver) struct Delays {}

impl delays::Delays for Delays {
    /// Get System Timer's counter
    fn get_timer_value() -> usize {
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

        ((hi as u64) << 32 | lo as u64) as usize
    }

    /// Wait N microsec (with BCM System Timer)
    fn wait_microsec(n: usize) {
        let t = Self::get_timer_value();
        // we must check if it's non-zero, because qemu does not emulate
        // system timer, and returning constant zero would mean infinite loop
        if t > 0 {
            while Self::get_timer_value() < t + n {}
        }
    }

    fn init() {}
}
