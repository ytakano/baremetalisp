use super::memory::*;
use crate::driver::{delays, uart::UART};
use core::ptr::{read_volatile, write_volatile};

const UART0_DR: *mut u32 = (MMIO_BASE + 0x00201000) as *mut u32;
const UART0_FR: *mut u32 = (MMIO_BASE + 0x00201018) as *mut u32;
const UART0_IBRD: *mut u32 = (MMIO_BASE + 0x00201024) as *mut u32;
const UART0_FBRD: *mut u32 = (MMIO_BASE + 0x00201028) as *mut u32;
const UART0_LCRH: *mut u32 = (MMIO_BASE + 0x0020102C) as *mut u32;
const UART0_CR: *mut u32 = (MMIO_BASE + 0x00201030) as *mut u32;
const UART0_IMSC: *mut u32 = (MMIO_BASE + 0x00201038) as *mut u32;
const UART0_ICR: *mut u32 = (MMIO_BASE + 0x00201044) as *mut u32;

pub(in crate::driver) struct RaspiUART {}

impl UART for RaspiUART {
    /// Initialiaze UART0 for serial console.
    /// Set baud rate and characteristics (8N1) and map to GPIO 14 (Tx) and 15 (Rx).
    /// 8N1 stands for "eight data bits, no parity, one stop bit".
    fn init(uart_clock: usize, baudrate: usize) {
        unsafe { write_volatile(UART0_CR, 0) }; // turn off UART0

        // map UART1 to GPIO pins
        let mut r = unsafe { read_volatile(GPFSEL1) };
        r &= !((7 << 12) | (7 << 15)); // gpio14, gpio15
        r |= (4 << 12) | (4 << 15); // alt0

        // enable pins 14 and 15
        unsafe {
            write_volatile(GPFSEL1, r);
            write_volatile(GPPUD, 0);
        }

        delays::wait_cycles(150);

        unsafe {
            write_volatile(GPPUDCLK0, (1 << 14) | (1 << 15));
        }

        delays::wait_cycles(150);

        let bauddiv: u32 = ((1000 * uart_clock) / (16 * baudrate)) as u32;
        let ibrd: u32 = bauddiv / 1000;
        let fbrd: u32 = ((bauddiv - ibrd * 1000) * 64 + 500) / 1000;

        unsafe {
            write_volatile(GPPUDCLK0, 0); // flush GPIO setup
            write_volatile(UART0_ICR, 0x7FF); // clear interrupts
            write_volatile(UART0_IBRD, ibrd);
            write_volatile(UART0_FBRD, fbrd);
            write_volatile(UART0_LCRH, 0b11 << 5); // 8n1
            write_volatile(UART0_CR, 0x301); // enable Tx, Rx, FIFO
        }
    }

    /// send a character to serial console
    fn send(c: u32) {
        // wait until we can send
        unsafe { asm!("nop;") };
        while unsafe { read_volatile(UART0_FR) } & 0x20 != 0 {
            unsafe { asm!("nop;") };
        }

        // write the character to the buffer
        unsafe {
            write_volatile(UART0_DR, c);
        }
    }

    fn recv() -> u32 {
        // wait until something is in the buffer
        unsafe { asm!("nop;") };
        while unsafe { read_volatile(UART0_FR) } & 0x10 != 0 {
            unsafe { asm!("nop;") };
        }

        // write the character to the buffer
        let c;
        unsafe {
            c = read_volatile(UART0_DR);
        }
        c as u32
    }
}

pub(in crate::driver) fn enable_recv_int() {
    todo!()
}
