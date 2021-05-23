pub mod defs;
pub mod delays;
mod device;
pub mod gic;
mod setup;
pub mod topology;
pub mod tzc380;
pub mod uart;

/// Initlize UART0 for serial console with 115200 8n1,
pub fn early_init() {
    delays::init();
    uart::init();
    uart::puts("\n");

    setup::early_platform_setup();
}

pub fn init() {
    setup::platform_setup();
}
