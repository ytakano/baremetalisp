mod cpu_on;
mod data;
pub mod ep_info;
mod setup;

use core::mem::size_of;

use crate::aarch64::{context, cpu};
use crate::driver;
use crate::driver::topology;

use ep_info::{Aapcs64Params, EntryPointInfo, ParamHeader};

pub(crate) type PsciResult = driver::psci::PsciResult;

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

pub const FUNCID_CC_SHIFT: u32 = 30;
pub const FUNCID_CC_MASK: u32 = 0x1;

// Flags and error codes
pub const SMC_64: u32 = 0;
pub const SMC_32: u32 = 1;

pub const SMC_TYPE_FAST: u32 = 1;
pub const SMC_TYPE_YIELD: u32 = 0;

// Various flags passed to SMC handlers
const SMC_FROM_SECURE: usize = 0 << 0;
const SMC_FROM_NON_SECURE: usize = 1 << 0;

extern "C" {
    fn ns_entry();
}

/// This function does the architectural setup and takes the warm boot
/// entry-point `mailbox_ep` as an argument. The function also initializes the
/// power domain topology tree by querying the platform. The power domain nodes
/// higher than the CPU are populated in the array psci_non_cpu_pd_nodes[] and
/// the CPU power domains are populated in psci_cpu_pd_nodes[]. The platform
/// exports its static topology map through the
/// populate_power_domain_topology_tree() API. The algorithm populates the
/// psci_non_cpu_pd_nodes and psci_cpu_pd_nodes iteratively by using this
/// topology map.  On a platform that implements two clusters of 2 cpus each,
/// and supporting 3 domain levels, the populated psci_non_cpu_pd_nodes would
/// look like this:
///
/// ---------------------------------------------------
/// | system node | cluster 0 node  | cluster 1 node  |
/// ---------------------------------------------------
///
/// And populated psci_cpu_pd_nodes would look like this :
/// <-    cpus cluster0   -><-   cpus cluster1   ->
/// ------------------------------------------------
/// |   CPU 0   |   CPU 1   |   CPU 2   |   CPU 3  |
/// ------------------------------------------------
pub fn init() {
    // Populate the power domain arrays using the platform topology map
    setup::populate_power_domain_tree(driver::topology::POWER_DOMAIN_TREE_DESC);

    // Update the CPU limits for each node in psci_non_cpu_pd_nodes */
    setup::update_pwrlvl_limits();

    // Populate the mpidr field of cpu node for this CPU
    data::set_cpu_pd_mpidr(
        topology::core_pos(),
        cpu::get_mpidr_el1() & cpu::MPIDR_AFFINITY_MASK,
    );

    setup::init_req_local_pwr_states();

    // Set the requested and target state of this CPU and all the higher
    // power domain levels for this CPU to run.
    setup::set_pwr_domains_to_run();

    // setup normal world's context
    let ep;
    let ptr = ns_entry as *const () as usize;
    match psci_validate_entry_point(ptr, 0) {
        Ok(e) => {
            ep = e;
        }
        Err(_) => {
            return;
        }
    }

    // Store the re-entry information for the non-secure world.
    context::init_context(topology::core_pos(), ep);
}

fn is_caller_non_secure(f: usize) -> bool {
    f & SMC_FROM_NON_SECURE != 0
}

fn is_caller_secure(f: usize) -> bool {
    !is_caller_non_secure(f)
}

/// PSCI top level handler for servicing SMCs.
pub fn psci_smc_handler(smc_fid: u32, x1: usize, x2: usize, x3: usize, flags: usize) -> PsciResult {
    if is_caller_secure(flags) {
        return PsciResult::PsciENotSupported;
    }

    if (smc_fid >> FUNCID_CC_SHIFT) & FUNCID_CC_MASK == SMC_32 {
        // AArch32
        match smc_fid {
            PSCI_CPU_ON_AARCH32 => psci_cpu_on(x1, x2, x3),
            _ => PsciResult::PsciENotSupported,
        }
    } else {
        // AArch64
        match smc_fid {
            PSCI_CPU_ON_AARCH64 => psci_cpu_on(x1, x2, x3),
            _ => PsciResult::PsciENotSupported,
        }
    }
}

fn psci_validate_mpidr(mpidr: usize) -> bool {
    match driver::topology::core_pos_by_mpidr(mpidr) {
        Some(_) => true,
        None => false,
    }
}

/// PSCI frontend api for servicing SMCs. Described in the PSCI spec.
fn psci_cpu_on(target_cpu: usize, entrypoint: usize, context_id: usize) -> PsciResult {
    // Determine if the cpu exists of not
    if psci_validate_mpidr(target_cpu) {
        return PsciResult::PsciEInvalidParams;
    }

    // Validate the entry point and get the entry_point_info
    let ep;
    match psci_validate_entry_point(entrypoint, context_id) {
        Ok(e) => {
            ep = e;
        }
        Err(e) => {
            return e;
        }
    }

    // To turn this cpu on, specify which power
    // levels need to be turned on
    cpu_on::psci_cpu_on_start(target_cpu, ep)
}

// This function validates the entrypoint with the platform layer if the
// appropriate pm_ops hook is exported by the platform and returns the
// 'entry_point_info'.
fn psci_validate_entry_point(
    entrypoint: usize,
    context_id: usize,
) -> Result<EntryPointInfo, PsciResult> {
    // Validate the entrypoint using platform psci_ops
    match driver::psci::validate_ns_entrypoint(entrypoint) {
        PsciResult::PsciESuccess => (),
        _ => {
            return Err(PsciResult::PsciEInvalidAddress);
        }
    }

    // Verify and derive the re-entry information for
    // the non-secure world from the non-secure state from
    // where this call originated.
    psci_get_ns_ep_info(entrypoint, context_id)
}

/// This function determines the full entrypoint information for the requested
/// PSCI entrypoint on power on/resume and returns it.
/// (for AArch64)
fn psci_get_ns_ep_info(entrypoint: usize, context_id: usize) -> Result<EntryPointInfo, PsciResult> {
    let ns_scr_el3 = cpu::get_scr_el3();
    let sctlr = if (ns_scr_el3 & cpu::SCR_HCE_BIT) != 0 {
        cpu::get_sctlr_el2()
    } else {
        cpu::get_sctlr_el1()
    };

    let ee;
    let ep_attr;
    if (sctlr & cpu::SCTLR_EE_BIT) != 0 {
        ep_attr = ep_info::EP_NON_SECURE | ep_info::EP_EE_BIG | ep_info::EP_ST_DISABLE;
        ee = 1;
    } else {
        ep_attr = ep_info::EP_NON_SECURE | ep_info::EP_ST_DISABLE;
        ee = 0;
    }

    // Figure out whether the cpu enters the non-secure address space
    // in aarch32 or aarch64
    let spsr = if (ns_scr_el3 & cpu::SCR_RW_BIT) != 0 {
        // Check whether a Thumb entry point has been provided for an
        // aarch64 EL
        if (entrypoint & 0x1) != 0 {
            return Err(PsciResult::PsciEInvalidAddress);
        }

        let mode = if (ns_scr_el3 & cpu::SCR_HCE_BIT) != 0 {
            cpu::EL::EL2h
        } else {
            cpu::EL::EL1h
        };

        cpu::spsr64(mode, cpu::DISABLE_ALL_EXCEPTIONS)
    } else {
        let mode = if (ns_scr_el3 & cpu::SCR_HCE_BIT) != 0 {
            cpu::MODE32_HYP
        } else {
            cpu::MODE32_SVC
        };

        // TODO: Choose async. exception bits if HYP mode is not
        // implemented according to the values of SCR.{AW, FW} bits
        let daif = cpu::DAIF_ABT_BIT | cpu::DAIF_IRQ_BIT | cpu::DAIF_FIQ_BIT;

        cpu::spsr32(mode, entrypoint as u64 & 1, ee, daif)
    };

    let headr = ParamHeader {
        htype: ep_info::PARAM_EP,
        version: ep_info::PARAM_VERSION_1,
        size: size_of::<ParamHeader>() as u16,
        attr: ep_attr as u32,
    };

    let mut args = Aapcs64Params::new();
    args.arg0 = context_id as u64;

    let ep = EntryPointInfo {
        h: headr,
        pc: entrypoint,
        spsr: spsr,
        args: args,
    };

    Ok(ep)
}
