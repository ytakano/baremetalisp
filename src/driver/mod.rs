use crate::aarch64;

pub mod uart;
pub mod memory;
pub mod mbox;
pub mod rand;
pub mod delays;
pub mod power;
pub mod graphics;

const UART_CLOCK: u64 = 48000000;
const UART_BAUD:  u64 = 115200;

pub struct Context {
    pub graphics0: Option<graphics::Display>,
    pub memory: usize
}

/// Initlize UART0 for serial console with 115200 8n1,
/// and graphics with 1024x768 resolution.
pub fn init() -> Context {
    uart::init(UART_CLOCK, UART_BAUD);

    aarch64::mmu::init();
    //rand::init();
    //uart::puts("initialized rand\n");

    let g = graphics::init();
    let m = mbox::get_memory();

    init_exceptions();

    Context{graphics0: g,
            memory: m}
}

fn init_exceptions() {
    ()
}
