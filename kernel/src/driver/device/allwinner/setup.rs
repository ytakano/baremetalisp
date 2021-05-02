use super::memory;
use crate::{
    aarch64::cpu,
    driver::{gic, tzc380},
    mmio::MMIO,
    print_hex32,
};

pub(in crate::driver) fn early_platform_setup() {}

const GICCDISABLE: u32 = 1 << 4;

pub(in crate::driver) fn platform_setup() {
    let ctrl = MMIO::new(memory::SUNXI_GENER_CTRL_REG0 as *mut u32);
    ctrl.clrbits(GICCDISABLE);
    cpu::dmb_st();

    let n = ctrl.read();
    print_hex32("CTRL REG0", n);

    gic::init(
        memory::SUNXI_GICC_BASE as usize,
        memory::SUNXI_GICD_BASE as usize,
        gic::GICVer::V2,
    );

    tzc380::init(memory::SUNXI_SMC_BASE as usize);
}
