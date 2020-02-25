use core::intrinsics::volatile_store;
use core::intrinsics::volatile_load;

use super::memory::*;
use super::mbox;
use super::delays;

const UART0_DR:   *mut u32 = (MMIO_BASE + 0x00201000) as *mut u32;
const UART0_FR:   *mut u32 = (MMIO_BASE + 0x00201018) as *mut u32;
const UART0_IBRD: *mut u32 = (MMIO_BASE + 0x00201024) as *mut u32;
const UART0_FBRD: *mut u32 = (MMIO_BASE + 0x00201028) as *mut u32;
const UART0_LCRH: *mut u32 = (MMIO_BASE + 0x0020102C) as *mut u32;
const UART0_CR:   *mut u32 = (MMIO_BASE + 0x00201030) as *mut u32;
const UART0_IMSC: *mut u32 = (MMIO_BASE + 0x00201038) as *mut u32;
const UART0_ICR:  *mut u32 = (MMIO_BASE + 0x00201044) as *mut u32;

// Set baud rate and characteristics (115200 8N1) and map to GPIO
// 8N1 stands for "eight data bits, no parity, one stop bit"
pub fn init() {
    unsafe { volatile_store(UART0_CR, 0) }; // turn off UART0

    // set up clock for consistent divisor values
    mbox::set_uart_clock(4000000); // 4Mhz

    // map UART1 to GPIO pins
    let mut r = unsafe { volatile_load(GPFSEL1) };
    r &= !((7 << 12) | (7 << 15));  // gpio14, gpio15
    r |=   (4 << 12) | (4 << 15);   // alt0

    // enable pins 14 and 15
    unsafe {
        volatile_store(GPFSEL1, r);
        volatile_store(GPPUD,   0);
    }

    delays::wait_cycles(150);

    unsafe {
        volatile_store(GPPUDCLK0, (1 << 14) | (1 << 15));
    }

    delays::wait_cycles(150);

    unsafe {
        volatile_store(GPPUDCLK0, 0);          // flush GPIO setup
        volatile_store(UART0_ICR, 0x7FF);      // clear interrupts
        volatile_store(UART0_IBRD, 2);         // 115200 baud
        volatile_store(UART0_FBRD, 0xB);
        volatile_store(UART0_LCRH, 0b11 << 5); // 8n1
        volatile_store(UART0_CR, 0x301);       // enable Tx, Rx, FIFO
    }
}

pub fn send(c : u32) {
    // wait until we can send
    unsafe { asm!("nop;") };
    while unsafe { volatile_load(UART0_FR) } & 0x20 != 0 {
        unsafe { asm!("nop;") };
    }

    // write the character to the buffer
    unsafe {
        volatile_store(UART0_DR, c);
    }
}

pub fn puts(s : &str) {
    for c in s.chars() {
        send(c as u32);
        if c == '\n' {
            send('\r' as u32);
        }
    }
}

pub fn hex(h : u64) {
    for i in (0..61).step_by(4).rev() {
        let mut n = (h >> i) & 0xF;
        n += if n > 9 { 0x37 } else { 0x30 };
        send(n as u32);
    }
}