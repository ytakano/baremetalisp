pub mod uart;
mod device;

/// Initlize UART0 for serial console with 115200 8n1,
pub fn init() {
    uart::init();

    //rand::init();
    //uart::puts("initialized rand\n");
}
