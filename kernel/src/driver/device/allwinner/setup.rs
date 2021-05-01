use super::memory;
use crate::{driver::gic, mmio::MMIO, print_hex32};

pub(in crate::driver) fn early_platform_setup() {}

const GICCDISABLE: u32 = 1 << 4;

pub(in crate::driver) fn platform_setup() {
    let ctrl = MMIO::new(memory::SUNXI_GENER_CTRL_REG0 as *mut u32);
    ctrl.clrbits(GICCDISABLE);

    let n = ctrl.read();
    print_hex32("CTRL_REG0", n);

    gic::init(
        memory::SUNXI_GICC_BASE as usize,
        memory::SUNXI_GICD_BASE as usize,
        gic::GICVer::V2,
    );
}
