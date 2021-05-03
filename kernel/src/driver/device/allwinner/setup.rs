use super::memory;
use crate::{
    aarch64::{cpu, mmu},
    driver::{gic, tzc380},
    mmio::MMIO,
    print_hex32,
};

pub(in crate::driver) fn early_platform_setup() {}

const GICCDISABLE: u32 = 1 << 4;

const SMC_MASTER_BYPASS: u32 = 0x18;
const SMC_MASTER_BYPASS_EN_MASK: u32 = 0x1;

pub(in crate::driver) fn platform_setup() {
    //init_gic();
    //init_smc();
}

/// initialize GIC
fn init_gic() {
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
}

/// enable TrustZone memory
fn init_smc() {
    tzc380::init(memory::SUNXI_SMC_BASE as usize);
    let tzc = tzc380::take().unwrap();
    tzc.configure_region(
        1,
        mmu::get_ram_start() as usize,
        tzc380::attr_region_size(tzc380::TZC_REGION_SIZE_32M)
            | tzc380::TZC_ATTR_REGION_EN_MASK
            | tzc380::TZC_ATTR_SP_S_RW,
    );

    let mb = MMIO::new((memory::SUNXI_SMC_BASE + SMC_MASTER_BYPASS) as *mut u32);
    mb.clrbits(SMC_MASTER_BYPASS_EN_MASK);
}
