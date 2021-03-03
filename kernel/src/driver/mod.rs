pub(crate) mod arm;
pub(crate) mod defs;
pub(crate) mod delays;
mod device;
pub(crate) mod memory;
pub(crate) mod psci;
mod setup;
pub(crate) mod topology;
pub(crate) mod uart;

#[cfg(feature = "pine64")]
mod mhu;

/// Initlize UART0 for serial console with 115200 8n1,
pub fn early_init() {
    delays::init();
    uart::init();
    uart::puts("\n");

    setup::early_platform_setup();

    //rand::init();
    //uart::puts("initialized rand\n");
}

pub fn init() {
    setup::platform_setup();
}
