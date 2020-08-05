use core::intrinsics::volatile_load;
use core::intrinsics::volatile_store;

use super::memory::{
    SUNXI_CPUCFG_BASE, SUNXI_R_CPUCFG_BASE, SUNXI_R_PRCM_BASE, SUNXI_SCP_BASE, SUNXI_SRAM_A2_BASE,
};
use crate::bits::{bit_clear32, bit_set32};
use crate::driver::arm::scpi;

const CPUCFG_DBG_REG0: *mut u32 = (SUNXI_CPUCFG_BASE + 0x0020) as *mut u32;
const SCP_FIRMWARE_MAGIC: u32 = 0xb4400012;
const OR1K_VEC_FIRST: u32 = 0x01;
const OR1K_VEC_LAST: u32 = 0x0e;

static mut SCPI_AVAILABLE: bool = false;

extern "C" {
    fn _start();
}

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

fn cpucfg_rvbar_lo_reg(core: usize) -> *mut u32 {
    (SUNXI_CPUCFG_BASE as usize + 0x00a0 + core * 8) as *mut u32
}

fn cpucfg_rvbar_hi_reg(core: usize) -> *mut u32 {
    (SUNXI_CPUCFG_BASE as usize + 0x00a4 + core * 8) as *mut u32
}

fn or1k_vec_addr(n: u32) -> u32 {
    0x100 * n
}

pub fn init() {
    // Program all CPU entry points
    let start = _start as *const () as u64;
    for i in 0..4 {
        let addr_lo = cpucfg_rvbar_lo_reg(i);
        let addr_hi = cpucfg_rvbar_hi_reg(i);
        unsafe {
            volatile_store(addr_lo, (start & 0xFFFFFFFF) as u32);
            volatile_store(addr_hi, (start >> 32) as u32);
        }
    }

    // Check for a valid SCP firmware, and boot the SCP if found.
    let scp_base = SUNXI_SCP_BASE as *mut u32;
    if unsafe { volatile_load(scp_base) } == SCP_FIRMWARE_MAGIC {
        // Program SCP exception vectors to the firmware entrypoint.
        for i in OR1K_VEC_FIRST..(OR1K_VEC_LAST + 1) {
            let vector = SUNXI_SRAM_A2_BASE + or1k_vec_addr(i);
            let offset = SUNXI_SCP_BASE - vector;
            unsafe {
                volatile_store(vector as *mut u32, offset >> 2);

                // TODO: clear cache
            }
            // Take the SCP out of reset.
            bit_set32(SUNXI_R_CPUCFG_BASE as *mut u32, 0);

            // Wait for the SCP firmware to boot.
            if scpi::scpi_wait_ready() {
                unsafe {
                    SCPI_AVAILABLE = true;
                }
            }
        }
    }

    if unsafe { SCPI_AVAILABLE } {
    } else {
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