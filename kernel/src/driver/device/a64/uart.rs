use core::intrinsics::volatile_load;
use core::intrinsics::volatile_store;

use super::memory::SUNXI_UART0_BASE;

pub const UART0_THR: *mut u64 = (SUNXI_UART0_BASE + 0x00) as *mut u64; // transmit holding register
pub const UART0_RBR: *mut u64 = (SUNXI_UART0_BASE + 0x00) as *mut u64; // receive holding register
pub const UART0_FCR: *mut u64 = (SUNXI_UART0_BASE + 0x08) as *mut u64; // fifo control register
pub const UART0_LSR: *mut u32 = (SUNXI_UART0_BASE + 0x14) as *mut u32; // line status register

pub fn init(_uart_clock: u64, _baudrate: u64) {
    unsafe {
        let val = volatile_load(UART0_FCR);
        volatile_store(UART0_THR, val | 1);
    }
}

/// send a character to serial console
pub fn send(c: u32) {
    while unsafe { volatile_load(UART0_LSR) } & (1 << 5) == 0 {
        unsafe { asm!("nop;") };
    }

    unsafe {
        volatile_store(UART0_THR, c as u64);
    }
}

pub fn recv() -> u32 {
    // wait until we can send
    unsafe { asm!("nop;") };
    while unsafe { volatile_load(UART0_LSR) } & 1 == 0 {
        unsafe { asm!("nop;") };
    }

    let c;
    unsafe {
        c = volatile_load(UART0_RBR);
    }
    c as u32
}
