mod arm;
mod device;
pub mod memory;
pub mod uart;

#[cfg(feature = "pine64")]
pub mod psci;

#[cfg(feature = "pine64")]
mod mhu;

/// Initlize UART0 for serial console with 115200 8n1,
pub fn init() {
    uart::init();

    //rand::init();
    //uart::puts("initialized rand\n");
}
