use core::ptr::{read_volatile, write_volatile};

use super::memory::SUNXI_UART0_BASE;
use crate::driver::uart::UART;

const UART0_THR: *mut u64 = (SUNXI_UART0_BASE + 0x00) as *mut u64; // transmit holding register
const UART0_RBR: *mut u64 = (SUNXI_UART0_BASE + 0x00) as *mut u64; // receive holding register
const UART0_IER: *mut u32 = (SUNXI_UART0_BASE + 0x04) as *mut u32; // interrupt enable register
const UART0_FCR: *mut u64 = (SUNXI_UART0_BASE + 0x08) as *mut u64; // fifo control register
const UART0_LSR: *mut u32 = (SUNXI_UART0_BASE + 0x14) as *mut u32; // line status register

pub(in crate::driver) struct A64UART {}

impl UART for A64UART {
    fn init(_uart_clock: usize, _baudrate: usize) {
        unsafe {
            let val = read_volatile(UART0_FCR);
            write_volatile(UART0_THR, val | 1);
        }
    }

    /// send a character to serial console
    fn send(c: u32) {
        while unsafe { read_volatile(UART0_LSR) } & (1 << 5) == 0 {
            unsafe { asm!("nop;") };
        }

        unsafe {
            write_volatile(UART0_THR, c as u64);
        }
    }

    fn recv() -> u32 {
        // wait until we can send
        unsafe { asm!("nop;") };
        while unsafe { read_volatile(UART0_LSR) } & 1 == 0 {
            unsafe { asm!("nop;") };
        }

        let c;
        unsafe {
            c = read_volatile(UART0_RBR);
        }
        c as u32
    }

    fn enable_recv_interrupt() {}
    fn disable_recv_interrupt() {}
}

pub(in crate::driver) fn enable_recv_int() {
    // enable received data available interrupt
    let ier = unsafe { read_volatile(UART0_IER) };
    crate::driver::uart::puts("UART0_IER = 0x");
    crate::driver::uart::hex32(ier);
    crate::driver::uart::puts("\n");

    unsafe { write_volatile(UART0_IER, 1) };

    let ier = unsafe { read_volatile(UART0_IER) };
    crate::driver::uart::puts("UART0_IER = 0x");
    crate::driver::uart::hex32(ier);
    crate::driver::uart::puts("\n");
}
