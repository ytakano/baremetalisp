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
}

/// Initlize UART0 for serial console with 115200 8n1,
/// and graphics with 1024x768 resolution.
pub fn init() -> Context {
    uart::init(UART_CLOCK, UART_BAUD);

    //rand::init();
    //uart::puts("initialized rand\n");

    let graphics0 = graphics::init();

    init_exceptions();

    Context{graphics0: graphics0}
}

fn init_exceptions() {
    ()
}
