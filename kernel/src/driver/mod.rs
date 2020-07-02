pub mod uart;
mod device;

const UART_BAUD: u64 = 115200;

/// Initlize UART0 for serial console with 115200 8n1,
/// and graphics with 1024x768 resolution.
pub fn init() {
    uart::init(UART_BAUD);

    //rand::init();
    //uart::puts("initialized rand\n");
}
