pub mod delays;
mod device;
pub mod gic;
pub mod topology;
pub mod tzc380;
pub mod uart;

/// Initlize UART0 for serial console with 115200 8n1,
pub fn early_init() {
    uart::init();
    uart::puts("\n");
}

pub fn init() {}
