use core::intrinsics::volatile_store;
use core::intrinsics::volatile_load;

use super::memory;

pub const SUNXI_UART0_BASE: u32 = (memory::MMIO_BASE + 0x00028000) as u32;

pub const UART0_THR: *mut u64 = (SUNXI_UART0_BASE + 0x00) as *mut u64; // transmit holding register
pub const UART0_FCR: *mut u64 = (SUNXI_UART0_BASE + 0x08) as *mut u64; // fifo control register
pub const UART0_LSR: *mut u32 = (SUNXI_UART0_BASE + 0x14) as *mut u32; // line status register

pub fn init() {
    unsafe {
        let val = volatile_load(UART0_FCR);
        volatile_store(UART0_THR, val | 1);
    }
}

/// send a character to serial console
pub fn send(c : u32) {
    while unsafe { volatile_load(UART0_LSR) } & (1 << 5) == 0 {
        unsafe { llvm_asm!("nop;") };
    }

    unsafe {
        volatile_store(UART0_THR, c as u64);
    }
}