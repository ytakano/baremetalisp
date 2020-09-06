mod cpu_on;
mod data;
pub mod ep_info;
mod setup;

use crate::aarch64::{context, cpu};
use crate::driver;
use crate::driver::psci::PsciResult;
use crate::driver::topology;

// Defines for runtime services function ids
pub const PSCI_VERSION: u32 = 0x84000000;
pub const PSCI_CPU_SUSPEND_AARCH32: u32 = 0x84000001;
pub const PSCI_CPU_SUSPEND_AARCH64: u32 = 0xc4000001;
pub const PSCI_CPU_OFF: u32 = 0x84000002;
pub const PSCI_CPU_ON_AARCH32: u32 = 0x84000003;
pub const PSCI_CPU_ON_AARCH64: u32 = 0xc4000003;
pub const PSCI_AFFINITY_INFO_AARCH32: u32 = 0x84000004;
pub const PSCI_AFFINITY_INFO_AARCH64: u32 = 0xc4000004;
pub const PSCI_MIG_AARCH32: u32 = 0x84000005;
pub const PSCI_MIG_AARCH64: u32 = 0xc4000005;
pub const PSCI_MIG_INFO_TYPE: u32 = 0x84000006;
pub const PSCI_MIG_INFO_UP_CPU_AARCH32: u32 = 0x84000007;
pub const PSCI_MIG_INFO_UP_CPU_AARCH64: u32 = 0xc4000007;
pub const PSCI_SYSTEM_OFF: u32 = 0x84000008;
pub const PSCI_SYSTEM_RESET: u32 = 0x84000009;
pub const PSCI_FEATURES: u32 = 0x8400000A;
pub const PSCI_NODE_HW_STATE_AARCH32: u32 = 0x8400000d;
pub const PSCI_NODE_HW_STATE_AARCH64: u32 = 0xc400000d;
pub const PSCI_SYSTEM_SUSPEND_AARCH32: u32 = 0x8400000E;
pub const PSCI_SYSTEM_SUSPEND_AARCH64: u32 = 0xc400000E;
pub const PSCI_STAT_RESIDENCY_AARCH32: u32 = 0x84000010;
pub const PSCI_STAT_RESIDENCY_AARCH64: u32 = 0xc4000010;
pub const PSCI_STAT_COUNT_AARCH32: u32 = 0x84000011;
pub const PSCI_STAT_COUNT_AARCH64: u32 = 0xc4000011;
pub const PSCI_SYSTEM_RESET2_AARCH32: u32 = 0x84000012;
pub const PSCI_SYSTEM_RESET2_AARCH64: u32 = 0xc4000012;
pub const PSCI_MEM_PROTECT: u32 = 0x84000013;
pub const PSCI_MEM_CHK_RANGE_AARCH32: u32 = 0x84000014;
pub const PSCI_MEM_CHK_RANGE_AARCH64: u32 = 0xc4000014;

pub const PSCI_FID_MASK: u32 = 0xffe0;
pub const PSCI_FID_VALUE: u32 = 0;

pub const FUNCID_CC_SHIFT: u32 = 30;
pub const FUNCID_CC_MASK: u32 = 0x1;

// Flags and error codes
pub const SMC_64: u32 = 1;
pub const SMC_32: u32 = 0;

pub const SMC_TYPE_FAST: u32 = 1;
pub const SMC_TYPE_YIELD: u32 = 0;

// Various flags passed to SMC handlers
const SMC_FROM_SECURE: usize = 0 << 0;
const SMC_FROM_NON_SECURE: usize = 1 << 0;

pub fn init() {
    setup::init();
}

pub fn init_warmboot() {
    setup::init_warmboot();
}

pub fn is_psci_fid(fid: u32) -> bool {
    (fid & PSCI_FID_MASK) == PSCI_FID_VALUE
}

/// PSCI top level handler for servicing SMCs.
pub fn smc_handler(smc_fid: u32, x1: usize, x2: usize, x3: usize) -> PsciResult {
    let is_secure = cpu::is_secure();
    if is_secure {
        return PsciResult::PsciENotSupported;
    }

    let ctx = context::get_ctx(topology::core_pos(), false);
    ctx.save_fpregs();

    let result = if (smc_fid >> FUNCID_CC_SHIFT) & FUNCID_CC_MASK == SMC_32 {
        // AArch32
        match smc_fid {
            PSCI_CPU_ON_AARCH32 => psci_cpu_on(x1, x2, x3),
            PSCI_SYSTEM_OFF => {
                driver::psci::system_off();
                PsciResult::PsciEInternFail
            }
            _ => PsciResult::PsciENotSupported,
        }
    } else {
        // AArch64
        match smc_fid {
            PSCI_CPU_ON_AARCH64 => psci_cpu_on(x1, x2, x3),
            _ => PsciResult::PsciENotSupported,
        }
    };

    ctx.restore_fpregs();

    result
}

fn validate_mpidr(mpidr: usize) -> bool {
    match driver::topology::core_pos_by_mpidr(mpidr) {
        Some(_) => true,
        None => false,
    }
}

/// PSCI frontend api for servicing SMCs. Described in the PSCI spec.
fn psci_cpu_on(target_cpu: usize, entrypoint: usize, context_id: usize) -> PsciResult {
    // Determine if the cpu exists of not
    if !validate_mpidr(target_cpu) {
        return PsciResult::PsciEInvalidParams;
    }

    // Validate the entry point and get the entry_point_info
    let ep;
    match setup::validate_entry_point(entrypoint, context_id) {
        Ok(e) => {
            ep = e;
        }
        Err(e) => {
            return e;
        }
    }

    // To turn this cpu on, specify which power
    // levels need to be turned on
    cpu_on::start(target_cpu, ep)
}
