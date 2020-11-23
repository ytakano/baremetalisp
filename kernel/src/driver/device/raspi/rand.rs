use core::ptr::{read_volatile, write_volatile};

use super::memory::*;

const RNG_CTRL: *mut u32 = (MMIO_BASE + 0x00104000) as *mut u32;
const RNG_STATUS: *mut u32 = (MMIO_BASE + 0x00104004) as *mut u32;
const RNG_DATA: *mut u32 = (MMIO_BASE + 0x00104008) as *mut u32;
const RNG_INT_MASK: *mut u32 = (MMIO_BASE + 0x00104010) as *mut u32;

/// Initilaize random number generator (only Raspberry Pi 3).
pub(in crate::driver) fn init() {
    unsafe {
        write_volatile(RNG_STATUS, 0x40000);

        // mask interrupt
        let mask = read_volatile(RNG_INT_MASK);
        write_volatile(RNG_INT_MASK, mask | 1);

        // enable
        let ctrl = read_volatile(RNG_CTRL);
        write_volatile(RNG_CTRL, ctrl | 1);
    }

    while (unsafe { read_volatile(RNG_STATUS) } >> 24) == 0 {
        unsafe { asm!("nop;") };
    }
}

pub(in crate::driver) fn rand32() -> u32 {
    unsafe { read_volatile(RNG_DATA) }
}

pub(in crate::driver) fn rand64() -> u64 {
    let v0 = rand32() as u64;
    let v1 = rand32() as u64;
    v0 | v1 << 32
}

pub(in crate::driver) fn rand_min_max32(min: u32, max: u32) -> u32 {
    rand32() % (max - min) + min
}

pub(in crate::driver) fn rand_min_max64(min: u64, max: u64) -> u64 {
    rand64() % (max - min) + min
}
