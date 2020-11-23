// Reduced Serial Bus
// https://linux-sunxi.org/Reduced_Serial_Bus

use core::ptr::{read_volatile, write_volatile};

use super::axp;
use super::defs;
use super::memory;
use crate::driver::uart;

pub(super) const RSB_CTRL: u32 = 0x00;
pub(super) const RSB_CCR: u32 = 0x04;
pub(super) const RSB_INTE: u32 = 0x08;
pub(super) const RSB_STAT: u32 = 0x0c;
pub(super) const RSB_DADDR0: u32 = 0x10;
pub(super) const RSB_DLEN: u32 = 0x18;
pub(super) const RSB_DATA0: u32 = 0x1c;
pub(super) const RSB_LCR: u32 = 0x24;
pub(super) const RSB_PMCR: u32 = 0x28;
pub(super) const RSB_CMD: u32 = 0x2c;
pub(super) const RSB_SADDR: u32 = 0x30;

pub(super) const RSBCMD_SRTA: u32 = 0xE8; // Set run-time address
pub(super) const RSBCMD_RD8: u32 = 0x8B;
pub(super) const RSBCMD_RD16: u32 = 0x9C;
pub(super) const RSBCMD_RD32: u32 = 0xA6;
pub(super) const RSBCMD_WR8: u32 = 0x4E;
pub(super) const RSBCMD_WR16: u32 = 0x59;
pub(super) const RSBCMD_WR32: u32 = 0x63;

pub(super) const MAX_TRIES: u32 = 100000;

fn wait_bit(desc: &str, offset: usize, mask: u32) -> bool {
    let ptr = (memory::SUNXI_R_RSB_BASE as usize + offset) as *const u32;
    let mut tries = MAX_TRIES;
    loop {
        let reg = unsafe { read_volatile(ptr) };
        if reg & mask == 0 {
            return true;
        }

        tries -= 1;
        if tries == 0 {
            // timed out
            uart::puts("error: ");
            uart::puts(desc);
            uart::puts(" timed out\n");
            return false;
        }
    }
}

fn wait_stat(desc: &str) -> bool {
    if !wait_bit(desc, RSB_CTRL as usize, 1 << 7) {
        return false;
    }

    let ptr = (memory::SUNXI_R_RSB_BASE + RSB_STAT) as *const u32;
    let reg = unsafe { read_volatile(ptr) };
    if reg == 0x01 {
        return true;
    }

    false
}

pub(super) fn init() -> bool {
    if !init_controller() {
        return false;
    }

    // Start with 400 KHz to issue the I2C->RSB switch command.
    if !set_bus_speed(defs::SYSCNT_FRQ, 400000) {
        return false;
    }

    // Initiate an I2C transaction to write 0x7c into register 0x3e,
    // switching the PMIC to RSB mode.
    if !set_device_mode(0x7c3e00) {
        return false;
    }

    // Now in RSB mode, switch to the recommended 3 MHz.
    if !set_bus_speed(defs::SYSCNT_FRQ, 3000000) {
        return false;
    }

    // Associate the 8-bit runtime address with the 12-bit bus address.
    if !assign_runtime_address(axp::AXP803_HW_ADDR, axp::AXP803_RT_ADDR) {
        return false;
    }

    axp::check_id()
}

/// Initialize the RSB controller.
fn init_controller() -> bool {
    let ptr = (memory::SUNXI_R_RSB_BASE + RSB_CTRL) as *mut u32;
    unsafe { write_volatile(ptr, 0x01) }; // soft reset
    wait_bit("RSB: reset controller", RSB_CTRL as usize, 1)
}

pub(super) fn set_bus_speed(source_req: u32, bus_freq: u32) -> bool {
    if bus_freq == 0 {
        return false;
    }

    let reg = source_req / bus_freq;
    if reg < 2 {
        return false;
    }

    let reg = reg / 2 - 1;
    let reg = reg | (1 << 8); // one cycle of CD output delay

    let ptr = (memory::SUNXI_R_RSB_BASE + RSB_CCR) as *mut u32;
    unsafe { write_volatile(ptr, reg) };

    true
}

pub(super) fn set_device_mode(device_mode: u32) -> bool {
    let ptr = (memory::SUNXI_R_RSB_BASE + RSB_PMCR) as *mut u32;
    unsafe { write_volatile(ptr, (device_mode & 0x00ffffff) | (1 << 31)) };
    wait_bit("RSB: set device to RSB", RSB_PMCR as usize, 1 << 31)
}

pub(super) fn assign_runtime_address(hw_addr: u32, rt_addr: u32) -> bool {
    unsafe {
        let ptr = (memory::SUNXI_R_RSB_BASE + RSB_SADDR) as *mut u32;
        write_volatile(ptr, (rt_addr << 16) | hw_addr);

        let ptr = (memory::SUNXI_R_RSB_BASE + RSB_CMD) as *mut u32;
        write_volatile(ptr, RSBCMD_SRTA);

        let ptr = (memory::SUNXI_R_RSB_BASE + RSB_CTRL) as *mut u32;
        write_volatile(ptr, 0x80);
    }

    wait_stat("RSB: set run-time address")
}

pub(super) fn read(rt_addr: u32, reg_addr: u32) -> Option<u32> {
    unsafe {
        let ptr = (memory::SUNXI_R_RSB_BASE + RSB_CMD) as *mut u32;
        write_volatile(ptr, RSBCMD_RD8); // read a byte

        let ptr = (memory::SUNXI_R_RSB_BASE + RSB_SADDR) as *mut u32;
        write_volatile(ptr, rt_addr << 16);

        let ptr = (memory::SUNXI_R_RSB_BASE + RSB_DADDR0) as *mut u32;
        write_volatile(ptr, reg_addr);

        let ptr = (memory::SUNXI_R_RSB_BASE + RSB_CTRL) as *mut u32;
        write_volatile(ptr, 0x80); // start transaction
    }

    if !wait_stat("RSB: read command") {
        return None;
    }

    let ptr = (memory::SUNXI_R_RSB_BASE + RSB_DATA0) as *const u32;
    Some(unsafe { read_volatile(ptr) & 0xff })
}

pub(super) fn write(rt_addr: u32, reg_addr: u32, value: u32) -> bool {
    unsafe {
        let ptr = (memory::SUNXI_R_RSB_BASE + RSB_CMD) as *mut u32;
        write_volatile(ptr, RSBCMD_WR8); // byte write

        let ptr = (memory::SUNXI_R_RSB_BASE + RSB_SADDR) as *mut u32;
        write_volatile(ptr, rt_addr << 16);

        let ptr = (memory::SUNXI_R_RSB_BASE + RSB_DADDR0) as *mut u32;
        write_volatile(ptr, reg_addr);

        let ptr = (memory::SUNXI_R_RSB_BASE + RSB_DATA0) as *mut u32;
        write_volatile(ptr, value);

        let ptr = (memory::SUNXI_R_RSB_BASE + RSB_CTRL) as *mut u32;
        write_volatile(ptr, 0x80); // start transaction
    }

    wait_stat("RSB: write command")
}
