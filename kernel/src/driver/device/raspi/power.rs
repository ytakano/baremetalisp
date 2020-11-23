use core::ptr::{read_volatile, write_volatile};

use super::mbox;
use super::memory::*;
use crate::driver::delays;

const PM_RSTC: *mut u32 = (MMIO_BASE + 0x0010001c) as *mut u32;
const PM_RSTS: *mut u32 = (MMIO_BASE + 0x00100020) as *mut u32;
const PM_WDOG: *mut u32 = (MMIO_BASE + 0x00100024) as *mut u32;
const PM_WDOG_MAGIC: u32 = 0x5a000000;
const PM_RSTC_FULLRST: u32 = 0x00000020;

/// Shutdown the board
pub(in crate::driver) fn shutdown() {
    // power off devices one by one
    for r in 0..16 {
        mbox::set_power_off(r);
    }

    // power off gpio pins (but not VCC pins)
    unsafe {
        write_volatile(GPFSEL0, 0);
        write_volatile(GPFSEL1, 0);
        write_volatile(GPFSEL2, 0);
        write_volatile(GPFSEL3, 0);
        write_volatile(GPFSEL4, 0);
        write_volatile(GPFSEL5, 0);
        write_volatile(GPPUD, 0);
    }

    delays::wait_cycles(150);

    unsafe {
        write_volatile(GPPUDCLK0, 0xffffffff);
        write_volatile(GPPUDCLK1, 0xffffffff);
    }

    delays::wait_cycles(150);

    unsafe {
        write_volatile(GPPUDCLK0, 0);
        write_volatile(GPPUDCLK1, 0);
    }

    // power off the SoC (GPU + CPU)
    let mut r = unsafe { read_volatile(PM_RSTS) };
    r &= !0xfffffaaa;
    r |= 0x555; // partition 63 used to indicate halt

    unsafe {
        write_volatile(PM_RSTS, PM_WDOG_MAGIC | r);
        write_volatile(PM_WDOG, PM_WDOG_MAGIC | 10);
        write_volatile(PM_RSTC, PM_WDOG_MAGIC | PM_RSTC_FULLRST);
    }
}

/// Reboot
pub(in crate::driver) fn reset() {
    // trigger a restart by instructing the GPU to boot from partition 0
    let mut r = unsafe { read_volatile(PM_RSTS) };
    r &= !0xfffffaaa;
    unsafe {
        write_volatile(PM_RSTS, PM_WDOG_MAGIC | r);
        write_volatile(PM_WDOG, PM_WDOG_MAGIC | 10);
        write_volatile(PM_RSTC, PM_WDOG_MAGIC | PM_RSTC_FULLRST);
    }
}
