mod arm;
mod device;
pub mod memory;
pub mod psci;
mod setup;
pub mod topology;
pub mod uart;

#[cfg(feature = "pine64")]
mod mhu;

/// Initlize UART0 for serial console with 115200 8n1,
pub fn early_init() {
    uart::init();

    //rand::init();
    //uart::puts("initialized rand\n");
}

pub fn init() {
    setup::platform_setup();
}
