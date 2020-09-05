use core::mem::size_of;
use core::ptr::{read_volatile, write_volatile};

use super::memory::{
    SUNXI_CPUCFG_BASE, SUNXI_R_CPUCFG_BASE, SUNXI_R_PRCM_BASE, SUNXI_SCP_BASE, SUNXI_SRAM_A2_BASE,
};
use crate::aarch64::{cache, lock};
use crate::bits::{bit_clear32, bit_set32};
use crate::driver::arm::scpi;
use crate::driver::{topology, uart};

const CPUCFG_DBG_REG0: *mut u32 = (SUNXI_CPUCFG_BASE + 0x0020) as *mut u32;
const SCP_FIRMWARE_MAGIC: u32 = 0xb4400012;
const OR1K_VEC_FIRST: u32 = 0x01;
const OR1K_VEC_LAST: u32 = 0x0e;

static mut SCPI_AVAILABLE: bool = false;

static mut ARISC_CORE_OFF: [u32; 27] = [
    0x18600000, // l.movhi  r3, <corenr>
    0x18000000, // l.movhi  r0, 0x0
    0x19a00170, // l.movhi  r13, 0x170
    0x84ad0030, // l.lwz    r5, 0x30(r13)
    0xe0a51803, // l.and    r5, r5, r3
    0xe4050000, // l.sfeq   r5, r0
    0x13fffffd, // l.bf     -12
    0xb8c30050, // l.srli   r6, r3, 16
    0xbc060001, // l.sfeqi  r6, 1
    0x10000005, // l.bf     +20
    0x19a001f0, // l.movhi  r13, 0x1f0
    0x84ad1500, // l.lwz    r5, 0x1500(r13)
    0xe0a53004, // l.or     r5, r5, r6
    0xd44d2d00, // l.sw     0x1500(r13), r5
    0x84ad1c30, // l.lwz    r5, 0x1c30(r13)
    0xacc6ffff, // l.xori   r6, r6, -1
    0xe0a53003, // l.and    r5, r5, r6
    0xd46d2c30, // l.sw     0x1c30(r13), r5
    0xe0c3000f, // l.ff1    r6, r3
    0x9cc6ffef, // l.addi   r6, r6, -17
    0xb8c60002, // l.slli   r6, r6, 2
    0xe0c66800, // l.add    r6, r6, r13
    0xa8a000ff, // l.ori    r5, r0, 0xff
    0xd4462d40, // l.sw     0x1540(r6), r5
    0xd46d0400, // l.sw     0x1c00(r13), r0
    0x03ffffff, // l.j      -1
    0x15000000, // l.nop
];

pub(crate) fn scpi_available() -> bool {
    unsafe { read_volatile(&SCPI_AVAILABLE) }
}

fn set_scpi_available(v: bool) {
    unsafe { write_volatile(&mut SCPI_AVAILABLE, v) }
}

extern "C" {
    fn _start();
}

fn cpucfg_cls_ctrl_reg0(cluster: usize) -> *mut u32 {
    (SUNXI_CPUCFG_BASE + (cluster as u32) * 16) as *mut u32
}

fn cpucfg_rst_ctrl_reg(cluster: usize) -> *mut u32 {
    (SUNXI_CPUCFG_BASE + 0x0080 + (cluster as u32) * 4) as *mut u32
}

fn poweron_rst_reg(cluster: usize) -> *mut u32 {
    (SUNXI_R_CPUCFG_BASE + 0x0030 + (cluster as u32) * 4) as *mut u32
}

fn poweroff_gating_reg(cluster: usize) -> *mut u32 {
    (SUNXI_R_PRCM_BASE + 0x0100 + (cluster as u32) * 4) as *mut u32
}

fn cpu_power_clamp_reg(cluster: usize, core: usize) -> *mut u32 {
    (SUNXI_R_PRCM_BASE as usize + 0x0140 + cluster * 16 + core * 4) as *mut u32
}

fn cpu_disable_power(cluster: usize, core: usize) {
    let ptr = cpu_power_clamp_reg(cluster, core);
    if unsafe { read_volatile(ptr) } == 0xff {
        return;
    }

    uart::puts("PSCI: Disabling power to cluster ");
    uart::decimal(cluster as u64);
    uart::puts(", core ");
    uart::decimal(core as u64);
    uart::puts("\n");

    unsafe { write_volatile(ptr, 0xff) };
}

fn enable_power(cluster: usize, core: usize) {
    let addr = cpu_power_clamp_reg(cluster, core);
    if unsafe { read_volatile(addr) } == 0 {
        return;
    }

    uart::puts("PSCI: Enabling power to cluster ");
    uart::decimal(cluster as u64);
    uart::puts(", core ");
    uart::decimal(core as u64);
    uart::puts("\n");

    // Power enable sequence from original Allwinner sources
    unsafe {
        write_volatile(addr, 0xfe);
        write_volatile(addr, 0xf8);
        write_volatile(addr, 0xe0);
        write_volatile(addr, 0x80);
        write_volatile(addr, 0x00);
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

pub(crate) fn init() {
    // Program all CPU entry points
    let start = _start as *const () as u64;
    for i in 0..4 {
        let addr_lo = cpucfg_rvbar_lo_reg(i);
        let addr_hi = cpucfg_rvbar_hi_reg(i);
        unsafe {
            write_volatile(addr_lo, (start & 0xFFFFFFFF) as u32);
            write_volatile(addr_hi, (start >> 32) as u32);
        }
    }

    // Check for a valid SCP firmware, and boot the SCP if found.
    let scp_base = SUNXI_SCP_BASE as *mut u32;
    if unsafe { read_volatile(scp_base) } == SCP_FIRMWARE_MAGIC {
        // Program SCP exception vectors to the firmware entrypoint.
        for i in OR1K_VEC_FIRST..(OR1K_VEC_LAST + 1) {
            let vector = SUNXI_SRAM_A2_BASE + or1k_vec_addr(i);
            let offset = SUNXI_SCP_BASE - vector;
            unsafe {
                write_volatile(vector as *mut u32, offset >> 2);
                cache::clean(&mut *(vector as *mut u32), size_of::<u32>());
            }
        }
        // Take the SCP out of reset.
        bit_set32(SUNXI_R_CPUCFG_BASE as *mut u32, 0);

        // Wait for the SCP firmware to boot.
        if scpi::wait_ready() {
            set_scpi_available(true);
        }
    } else {
        set_scpi_available(false);
    }
}

pub(crate) fn cpu_on(mpidr: usize) {
    let cluster = (mpidr >> 8) & 0xFF;
    let core = mpidr & 0xFF;

    uart::puts("PSCI: Powering on cluster ");
    uart::decimal(cluster as u64);
    uart::puts(", core ");
    uart::decimal(core as u64);
    uart::puts("\n");

    let cls_ctrl = cpucfg_cls_ctrl_reg0(cluster);
    let rst_ctrl = cpucfg_rst_ctrl_reg(cluster);
    let poweron_rst = poweron_rst_reg(cluster);
    let poweroff_gating = poweroff_gating_reg(cluster);

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

pub(crate) fn cpu_off(mpidr: usize) {
    let cluster = (mpidr >> 8) & 0xFF;
    let core = mpidr & 0xFF;

    uart::puts("PSCI: Powering off cluster ");
    uart::decimal(cluster as u64);
    uart::puts(", core ");
    uart::decimal(core as u64);
    uart::puts("\n");

    // Deassert DBGPWRDUP
    bit_clear32(CPUCFG_DBG_REG0, core as u32);

    let idx;
    match topology::core_pos_by_mpidr(mpidr) {
        Some(x) => {
            idx = x;
        }
        None => {
            return;
        }
    }

    // We can't turn ourself off like this, but it works for other cores
    if topology::core_pos() != idx {
        // Activate the core output clamps, but not for core 0.
        if core != 0 {
            bit_set32(poweroff_gating_reg(cluster), core as u32);
        }

        // Assert CPU power-on reset
        bit_clear32(poweron_rst_reg(cluster), core as u32);
        // Remove power from the CPU
        cpu_disable_power(cluster, core);

        return;
    }

    // If we are supposed to turn ourself off, tell the arisc SCP
    // to do that work for us. The code expects the core mask to be
    // patched into the first instruction.
    sunxi_execute_arisc_code(unsafe { &mut ARISC_CORE_OFF }, 1 << core);
}

static mut ARISC_LOCK: lock::BakeryTicket = lock::BakeryTicket::new();

/// Tell the "arisc" SCP core (an OpenRISC core) to execute some code.
/// We don't have any service running there, so we place some OpenRISC code
/// in SRAM, put the address of that into the reset vector and release the
/// arisc reset line. The SCP will execute that code and pull the line up again.
fn sunxi_execute_arisc_code(code: &mut [u32], param: u32) {
    let arisc_reset_vec = (SUNXI_SRAM_A2_BASE + 0x100) as *mut u32;

    let _ = unsafe { ARISC_LOCK.lock() };

    loop {
        // Wait until the arisc is in reset state.
        if unsafe { read_volatile(SUNXI_R_CPUCFG_BASE as *mut u32) } & 1 != 0 {
            break;
        }
    }

    // Patch up the code to feed in an input parameter.
    code[0] = (code[0] & !0xffff) | param;
    cache::clean(&code[0], code.len() * size_of::<u32>());

    // The OpenRISC unconditional branch has opcode 0, the branch offset
    // is in the lower 26 bits, containing the distance to the target,
    // in instruction granularity (32 bits).
    let code_addr = &code[0] as *const u32 as u32;
    let reset_vec_addr = SUNXI_SRAM_A2_BASE + 0x100;
    unsafe { write_volatile(arisc_reset_vec, (code_addr - reset_vec_addr) / 4) };
    cache::clean(unsafe { &(*arisc_reset_vec) }, 4);

    // De-assert the arisc reset line to let it run.
    bit_set32(SUNXI_R_CPUCFG_BASE as *mut u32, 0);
}

pub fn disable_secondary_cpus(primary_mpidr: usize) {
    for cluster in 0..topology::CLUSTER_COUNT {
        for core in 0..topology::MAX_CPUS_PER_CLUSTER {
            let mpidr = (cluster << 8) | core | (1 << 31);
            if mpidr != primary_mpidr {
                cpu_off(mpidr);
            }
        }
    }
}
