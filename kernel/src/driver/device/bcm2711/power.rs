use core::intrinsics::volatile_store;
use core::intrinsics::volatile_load;

use super::memory::*;
use super::mbox;
use super::delays;

const PM_RSTC:         *mut u32 = (MMIO_BASE + 0x0010001c) as *mut u32;
const PM_RSTS:         *mut u32 = (MMIO_BASE + 0x00100020) as *mut u32;
const PM_WDOG:         *mut u32 = (MMIO_BASE + 0x00100024) as *mut u32;
const PM_WDOG_MAGIC:   u32 = 0x5a000000;
const PM_RSTC_FULLRST: u32 = 0x00000020;

/// Shutdown the board
pub fn shutdown() {
    // power off devices one by one
    for r in 0..16 {
        mbox::set_power_off(r);
    }

    // power off gpio pins (but not VCC pins)
    unsafe {
        volatile_store(GPFSEL0, 0);
        volatile_store(GPFSEL1, 0);
        volatile_store(GPFSEL2, 0);
        volatile_store(GPFSEL3, 0);
        volatile_store(GPFSEL4, 0);
        volatile_store(GPFSEL5, 0);
        volatile_store(GPPUD, 0);
    }

    delays::wait_cycles(150);

    unsafe {
        volatile_store(GPPUDCLK0, 0xffffffff);
        volatile_store(GPPUDCLK1, 0xffffffff);
    }

    delays::wait_cycles(150);

    unsafe {
        volatile_store(GPPUDCLK0, 0);
        volatile_store(GPPUDCLK1, 0);
    }

    // power off the SoC (GPU + CPU)
    let mut r = unsafe { volatile_load(PM_RSTS) };
    r &= !0xfffffaaa;
    r |= 0x555; // partition 63 used to indicate halt

    unsafe {
        volatile_store(PM_RSTS, PM_WDOG_MAGIC | r);
        volatile_store(PM_WDOG, PM_WDOG_MAGIC | 10);
        volatile_store(PM_RSTC, PM_WDOG_MAGIC | PM_RSTC_FULLRST);
    }
}

/// Reboot
pub fn reset() {
    // trigger a restart by instructing the GPU to boot from partition 0
    let mut r = unsafe { volatile_load(PM_RSTS) };
    r &= !0xfffffaaa;
    unsafe {
        volatile_store(PM_RSTS, PM_WDOG_MAGIC | r);
        volatile_store(PM_WDOG, PM_WDOG_MAGIC | 10);
        volatile_store(PM_RSTC, PM_WDOG_MAGIC | PM_RSTC_FULLRST);
    }
}