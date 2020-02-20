use core::intrinsics::volatile_store;
use core::intrinsics::volatile_load;

use super::memory::*;

pub fn init() -> () {
    let baud = (SYS_FREQ / ( 8 * BAUD)) - 1;
    unsafe {
        volatile_store(AUX_ENABLE, *AUX_ENABLE | 1); // enable UART1, AUX mini uart
        volatile_store(AUX_MU_CNTL, 0);
        volatile_store(AUX_MU_LCR,  3);    // 8bits
        volatile_store(AUX_MU_MCR,  0);
        volatile_store(AUX_MU_IER,  0);
        volatile_store(AUX_MU_IIR,  0xc6); // disable interrupt
        volatile_store(AUX_MU_BAUD, baud); // 115200 baud
    };

    // map UART1 to GPIO pins
    let mut r = unsafe { volatile_load(GPFSEL1) };
    r &= !((7 << 12) | (7 << 15));  // gpio14, gpio15
    r |=   (2 << 12) | (2 << 15);   // alt5

    unsafe {
        volatile_store(GPFSEL1, r);
        volatile_store(GPPUD,   0); // enable pins 14 and 15
    };

    for _ in 0..150 {
        unsafe { asm!("nop;") };
    };

    unsafe {
        volatile_store(GPPUDCLK0, (1 << 14) | (1 << 15));
    };

    for _ in 0..150 {
        unsafe { asm!("nop;") };
    };

    unsafe {
        volatile_store(GPPUDCLK0,   0); // flush GPIO setup
        volatile_store(AUX_MU_CNTL, 3); // enable Tx, Rx
    };

    ()
}

pub fn send(c : u32) -> () {
    // wait until we can send
    while unsafe { volatile_load(AUX_MU_LSR) } & 1 == 1 {
        unsafe { asm!("nop;") };
    };

    // write the character to the buffer
    unsafe {
        volatile_store(AUX_MU_IO, c);
    };

    ()
}

pub fn puts(s : &str) -> () {
    for c in s.chars() {
        send(c as u32);
        if c == '\n' {
            send('\r' as u32);
        }
    };

    ()
}