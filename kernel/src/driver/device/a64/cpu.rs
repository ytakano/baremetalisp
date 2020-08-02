use core::intrinsics::volatile_load;
use core::intrinsics::volatile_store;

use super::memory::{SUNXI_CPUCFG_BASE, SUNXI_R_CPUCFG_BASE, SUNXI_R_PRCM_BASE};
use crate::bits::{bit_clear32, bit_set32};

const CPUCFG_DBG_REG0: *mut u32 = (SUNXI_CPUCFG_BASE + 0x0020) as *mut u32;
const CPU0_ADDR_L: *mut u32 = (SUNXI_CPUCFG_BASE + 0x00A0) as *mut u32;
const CPU0_ADDR_H: *mut u32 = (SUNXI_CPUCFG_BASE + 0x00A4) as *mut u32;
const CPU1_ADDR_L: *mut u32 = (SUNXI_CPUCFG_BASE + 0x00A8) as *mut u32;
const CPU1_ADDR_H: *mut u32 = (SUNXI_CPUCFG_BASE + 0x00AC) as *mut u32;
const CPU2_ADDR_L: *mut u32 = (SUNXI_CPUCFG_BASE + 0x00B0) as *mut u32;
const CPU2_ADDR_H: *mut u32 = (SUNXI_CPUCFG_BASE + 0x00B4) as *mut u32;
const CPU3_ADDR_L: *mut u32 = (SUNXI_CPUCFG_BASE + 0x00B8) as *mut u32;
const CPU3_ADDR_H: *mut u32 = (SUNXI_CPUCFG_BASE + 0x00BC) as *mut u32;

fn cpucfg_cls_ctrl_reg0(core: usize) -> *mut u32 {
    (SUNXI_CPUCFG_BASE + (core as u32) * 16) as *mut u32
}

fn cpucfg_rst_ctrl_reg(core: usize) -> *mut u32 {
    (SUNXI_CPUCFG_BASE + 0x0080 + (core as u32) * 4) as *mut u32
}

fn poweron_rst_reg(core: usize) -> *mut u32 {
    (SUNXI_R_CPUCFG_BASE + 0x0030 + (core as u32) * 4) as *mut u32
}

fn poweroff_gating_reg(core: usize) -> *mut u32 {
    (SUNXI_R_PRCM_BASE + 0x0100 + (core as u32) * 4) as *mut u32
}

fn cpu_power_clamp_reg(cluster: usize, core: usize) -> *mut u32 {
    (SUNXI_R_PRCM_BASE as usize + 0x0140 + cluster * 16 + core * 4) as *mut u32
}

fn enable_power(cluster: usize, core: usize) {
    let addr = cpu_power_clamp_reg(cluster, core);
    if unsafe { volatile_load(addr) } == 0 {
        return;
    }

    unsafe {
        volatile_store(addr, 0xfe);
        volatile_store(addr, 0xf8);
        volatile_store(addr, 0xe0);
        volatile_store(addr, 0x80);
        volatile_store(addr, 0x00);
    }
}

pub fn cpu_on(mpidr: usize) {
    let cluster = (mpidr >> 8) & 0xFF;
    let core = mpidr & 0xFF;

    let cls_ctrl = cpucfg_cls_ctrl_reg0(core);
    let rst_ctrl = cpucfg_rst_ctrl_reg(core);
    let poweron_rst = poweron_rst_reg(core);
    let poweroff_gating = poweroff_gating_reg(core);

    // Assert CPU core reset
    bit_clear32(rst_ctrl, core as u32);
    // Assert CPU power-on reset
    bit_clear32(poweron_rst, core as u32);
    // Set CPU to start in AArch64 mode
    bit_set32(cls_ctrl, 24 + core as u32);
    // Apply power to the CPU
    enable_power(cluster, core);
    // Release the core output clamps
    bit_clear32(poweroff_gating, core as u32);
    // Deassert CPU power-on reset
    bit_set32(poweron_rst, core as u32);
    // Deassert CPU core reset
    bit_set32(rst_ctrl, core as u32);
    // Assert DBGPWRDUP
    bit_set32(CPUCFG_DBG_REG0, core as u32);
}
