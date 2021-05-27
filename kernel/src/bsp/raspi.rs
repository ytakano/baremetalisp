pub(super) mod int;
pub(super) mod memory;

use super::{raspi::memory::*, BSPInit};
use crate::{
    bsp,
    driver::uart::{pl011::PL011, UART},
};

#[cfg(feature = "raspi3")]
pub const SYSCNT_FRQ: u32 = 19200000;

#[cfg(feature = "raspi4")]
pub const SYSCNT_FRQ: u32 = 54000000;

const UART0_BASE: usize = memory::MMIO_BASE + 0x00201000;
const UART0_CLOCK: usize = 48000000;
const UART0_BAUD: usize = 115200;

pub(super) struct Init {}

impl BSPInit for Init {
    fn early_init() {
        init_uart0();
    }

    fn init() {}
}

fn init_uart0() {
    let uart0 = PL011::new(UART0_BASE);
    uart0.off();

    // map UART1 to GPIO pins
    let mut r = gpfsel1().read();
    r &= !((7 << 12) | (7 << 15)); // gpio14, gpio15
    r |= (4 << 12) | (4 << 15); // alt0

    // enable pins 14 and 15
    gpfsel1().write(r);
    gppud().write(0);

    bsp::delays::wait_cycles(150);

    gppudclk0().write((1 << 14) | (1 << 15));

    bsp::delays::wait_cycles(150);

    gppudclk0().write(0); // flush GPIO setup

    uart0.init(UART0_CLOCK, UART0_BAUD);
    uart0.on();

    super::uart::init(uart0);
}
