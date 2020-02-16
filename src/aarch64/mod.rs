use core::intrinsics::volatile_store;

/* Define the system clock frequency in MHz for the baud rate calculation.
   This is clearly defined on the BCM2835 datasheet errata page:
   http://elinux.org/BCM2835_datasheet_errata */
pub const SYS_FREQ: u32 = 250000000;
pub const BAUD:     u32 = 115200;

#[cfg(any(feature = "rspi3", feature = "rspi2"))]
pub const MMIO_BASE: u32 = 0x3F000000;

#[cfg(feature = "rspi4")]
pub const MMIO_BASE: u32 = 0xFE000000;

pub const GPFSEL0:   *mut u32 = (MMIO_BASE + 0x00200000) as *mut u32;
pub const GPFSEL1:   *mut u32 = (MMIO_BASE + 0x00200004) as *mut u32;
pub const GPFSEL2:   *mut u32 = (MMIO_BASE + 0x00200008) as *mut u32;
pub const GPFSEL3:   *mut u32 = (MMIO_BASE + 0x0020000C) as *mut u32;
pub const GPFSEL4:   *mut u32 = (MMIO_BASE + 0x00200010) as *mut u32;
pub const GPFSEL5:   *mut u32 = (MMIO_BASE + 0x00200014) as *mut u32;
pub const GPSET0:    *mut u32 = (MMIO_BASE + 0x0020001C) as *mut u32;
pub const GPSET1:    *mut u32 = (MMIO_BASE + 0x00200020) as *mut u32;
pub const GPCLR0:    *mut u32 = (MMIO_BASE + 0x00200028) as *mut u32;
pub const GPLEV0:    *mut u32 = (MMIO_BASE + 0x00200034) as *mut u32;
pub const GPLEV1:    *mut u32 = (MMIO_BASE + 0x00200038) as *mut u32;
pub const GPEDS0:    *mut u32 = (MMIO_BASE + 0x00200040) as *mut u32;
pub const GPEDS1:    *mut u32 = (MMIO_BASE + 0x00200044) as *mut u32;
pub const GPHEN0:    *mut u32 = (MMIO_BASE + 0x00200064) as *mut u32;
pub const GPHEN1:    *mut u32 = (MMIO_BASE + 0x00200068) as *mut u32;
pub const GPPUD:     *mut u32 = (MMIO_BASE + 0x00200094) as *mut u32;
pub const GPPUDCLK0: *mut u32 = (MMIO_BASE + 0x00200098) as *mut u32;
pub const GPPUDCLK1: *mut u32 = (MMIO_BASE + 0x0020009C) as *mut u32;

pub const AUX_ENABLE:     *mut u32 = (MMIO_BASE + 0x00215004) as *mut u32;
pub const AUX_MU_IO:      *mut u32 = (MMIO_BASE + 0x00215040) as *mut u32;
pub const AUX_MU_IER:     *mut u32 = (MMIO_BASE + 0x00215044) as *mut u32;
pub const AUX_MU_IIR:     *mut u32 = (MMIO_BASE + 0x00215048) as *mut u32;
pub const AUX_MU_LCR:     *mut u32 = (MMIO_BASE + 0x0021504C) as *mut u32;
pub const AUX_MU_MCR:     *mut u32 = (MMIO_BASE + 0x00215050) as *mut u32;
pub const AUX_MU_LSR:     *mut u32 = (MMIO_BASE + 0x00215054) as *mut u32;
pub const AUX_MU_MSRL:    *mut u32 = (MMIO_BASE + 0x00215058) as *mut u32;
pub const AUX_MU_SCRATCH: *mut u32 = (MMIO_BASE + 0x0021505C) as *mut u32;
pub const AUX_MU_CNTL:    *mut u32 = (MMIO_BASE + 0x00215060) as *mut u32;
pub const AUX_MU_STAT:    *mut u32 = (MMIO_BASE + 0x00215064) as *mut u32;
pub const AUX_MU_BAUD:    *mut u32 = (MMIO_BASE + 0x00215068) as *mut u32;

pub fn init() -> () {
    uart_init();
    init_exceptions()
}

fn uart_init() -> () {
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
    let mut r = unsafe { *GPFSEL1 };
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

pub fn uart_send(c : u32) -> () {
    // wait until we can send
    while unsafe { *AUX_MU_LSR & 1 } == 1 {
        unsafe { asm!("nop;") };
    };

    // write the character to the buffer
    unsafe {
        volatile_store(AUX_MU_IO, c);
    };

    ()
}

pub fn uart_puts(s : &str) -> () {
    for c in s.chars() {
        uart_send(c as u32);
        if c == '\n' {
            uart_send('\r' as u32);
        }
    };

    ()
}

fn init_exceptions() -> () {

    ()
}
