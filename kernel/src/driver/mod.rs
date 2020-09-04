pub(crate) mod arm;
pub mod defs;
pub mod delays;
mod device;
pub mod memory;
pub mod psci;
mod setup;
pub mod topology;
pub mod uart;

#[cfg(feature = "pine64")]
mod mhu;

use crate::aarch64::cpu;

/// Initlize UART0 for serial console with 115200 8n1,
pub fn early_init() {
    // Program the counter frequency
    if cpu::get_current_el() == 3 {
        cpu::cntfrq_el0::set(defs::SYSCNT_FRQ as u64);
    }

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
