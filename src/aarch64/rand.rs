use core::intrinsics::volatile_store;
use core::intrinsics::volatile_load;

use super::memory::*;

const RNG_CTRL:     *mut u32 = (MMIO_BASE + 0x00104000) as *mut u32;
const RNG_STATUS:   *mut u32 = (MMIO_BASE + 0x00104004) as *mut u32;
const RNG_DATA:     *mut u32 = (MMIO_BASE + 0x00104008) as *mut u32;
const RNG_INT_MASK: *mut u32 = (MMIO_BASE + 0x00104010) as *mut u32;

/// Initilaize random number generator (only Raspberry Pi 3).
pub fn init() {
    unsafe {
        volatile_store(RNG_STATUS, 0x40000);

        // mask interrupt
        let mask = volatile_load(RNG_INT_MASK);
        volatile_store(RNG_INT_MASK, mask | 1);

        // enable
        let ctrl = volatile_load(RNG_CTRL);
        volatile_store(RNG_CTRL, ctrl | 1);
    }

    while (unsafe { volatile_load(RNG_STATUS) } >> 24) == 0 {
        unsafe { asm!("nop;") };
    }
}

pub fn rand32() -> u32 {
    unsafe { volatile_load(RNG_DATA) }
}

pub fn rand64() -> u64 {
    let v0 = rand32() as u64;
    let v1 = rand32() as u64;
    v0 | v1 << 32
}

pub fn rand_min_max32(min: u32, max: u32) -> u32 {
    rand32() % (max - min) + min
}

pub fn rand_min_max64(min: u64, max: u64) -> u64 {
    rand64() % (max - min) + min
}