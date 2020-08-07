mod cpu_on;
mod setup;

use crate::driver;

type PsciResult = driver::psci::PsciResult;

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

/// This structure provides version information and the size of the
/// structure, attributes for the structure it represents
pub struct ParamHeader {
    pub htype: u8,   // type of the structure
    pub version: u8, // version of this structure
    pub size: u16,   // size of this structure in bytes
    pub attr: u32,   // attributes: unused bits SBZ
}

pub struct PsciLibArgs {
    pub h: ParamHeader,
    pub mailbox_ep: usize,
}

pub struct Aapcs64Params {
    pub arg0: u64,
    pub arg1: u64,
    pub arg2: u64,
    pub arg3: u64,
    pub arg4: u64,
    pub arg5: u64,
    pub arg6: u64,
    pub arg7: u64,
}

pub struct Aapcs32Params {
    pub arg0: u32,
    pub arg1: u32,
    pub arg2: u32,
    pub arg3: u32,
}

// This structure represents the superset of information needed while
// switching exception levels. The only two mechanisms to do so are
// ERET & SMC. Security state is indicated using bit zero of header
// attribute
// NOTE: BL1 expects entrypoint followed by spsr at an offset from the start
// of this structure defined by the macro `ENTRY_POINT_INFO_PC_OFFSET` while
// processing SMC to jump to BL31.
pub struct EntryPointInfo {
    pub h: ParamHeader,
    pub pc: usize,
    pub spsr: u32,
    pub args: EPIArgs,
}

pub enum EPIArgs {
    AArch32(usize, Aapcs32Params), // lr_svc, args
    AArch64(Aapcs64Params),        // args
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
pub fn init() {}

/// PSCI top level handler for servicing SMCs.
pub fn psci_smc_handler(smc_fid: u32, x1: usize, x2: usize, x3: usize, flags: usize) -> PsciResult {
    if (smc_fid >> FUNCID_CC_SHIFT) & FUNCID_CC_MASK == SMC_32 {
        // AArch32
        PsciResult::PsciENotSupported
    } else {
        // AArch64
        match smc_fid {
            PSCI_CPU_ON_AARCH64 => psci_cpu_on(x1, x2, x3),
            _ => PsciResult::PsciENotSupported,
        }
    }
}

/// PSCI frontend api for servicing SMCs. Described in the PSCI spec.
fn psci_cpu_on(target_cpu: usize, entrypoint: usize, context_id: usize) -> PsciResult {
    PsciResult::PsciENotSupported
}
