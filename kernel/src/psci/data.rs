use core::mem::size_of;
use core::mem::transmute;
use core::ptr::{read_volatile, write_volatile};
use synctools::mcs::{MCSLock, MCSLockGuard};

use crate::aarch64::cache;
use crate::driver::defs;
use crate::driver::topology;

pub(super) const INVALID_MPIDR: u64 = !0;
pub(super) const INVALID_PWR_LVL: u8 = defs::MAX_PWR_LVL + 1;

// This is the power level corresponding to a CPU
pub(super) const PSCI_CPU_PWR_LVL: u8 = 0;

// The maximum power level supported by PSCI. Since PSCI CPU_SUSPEND
// uses the old power_state parameter format which has 2 bits to specify the
// power level, this constant is defined to be 3.
pub(super) const PSCI_MAX_PWR_LVL: u8 = 3;

// The local state macro used to represent RUN state.
pub(super) const PSCI_LOCAL_STATE_RUN: u8 = 0;

/// The following two data structures implement the power domain tree. The tree
/// is used to track the state of all the nodes i.e. power domain instances
/// described by the platform. The tree consists of nodes that describe CPU power
/// domains i.e. leaf nodes and all other power domains which are parents of a
/// CPU power domain i.e. non-leaf nodes.
pub(super) struct NonCpuPwrDomainNode {
    // Index of the first CPU power domain node level 0 which has this node
    // as its parent.
    cpu_start_idx: usize,

    // Number of CPU power domains which are siblings of the domain indexed
    // by 'cpu_start_idx' i.e. all the domains in the range 'cpu_start_idx
    // -> cpu_start_idx + ncpus' have this node as their parent.
    ncpus: usize,

    // Index of the parent power domain node.
    // TODO: Figure out whether to whether using pointer is more efficient.
    parent_node: usize,

    local_state: u8,
    level: u8,

    // For indexing the psci_lock array
    lock_var: MCSLock<()>,
}

pub(super) struct CpuPwrDomainNode {
    mpidr: u64,

    // Index of the parent power domain node.
    // TODO: Figure out whether to whether using pointer is more efficient.
    parent_node: usize,

    // A CPU power domain does not require state coordination like its
    // parent power domains. Hence this node does not include a bakery
    // lock. A spinlock is required by the CPU_ON handler to prevent a race
    // when multiple CPUs try to turn ON the same target CPU.
    cpu_lock: MCSLock<()>,
}

/// Structure used to store per-cpu information relevant to the PSCI service.
/// It is populated in the per-cpu data array. In return we get a guarantee that
/// this information will not reside on a cache line shared with another cpu.
struct PsciCpuData {
    // State as seen by PSCI Affinity Info API
    aff_info_state: AffInfoState,

    // Highest power level which takes part in a power management
    // operation.
    target_pwrlvl: u8,

    // The local power state of this CPU
    local_state: u8,
}

macro_rules! def_static {
    ($id:ident: [$t:ty; $n:expr]) => {
        static mut $id: [$t; $n] = unsafe {
            transmute::<[u8; size_of::<[$t; $n]>()], [$t; $n]>([0; size_of::<[$t; $n]>()])
        };
    };
}

// These are the states reported by the PSCI_AFFINITY_INFO API for the specified

// CPU. The definitions of these states can be found in Section 5.7.1 in the
// PSCI specification (ARM DEN 0022C).
pub(super) enum AffInfoState {
    StateOn = 0,
    StateOff,
    StateOnPending,
}

def_static!(NON_CPU_PD_NODES: [NonCpuPwrDomainNode; topology::NUM_NON_CPU_PWR_DOMAINS]);
def_static!(CPU_PD_NODES: [CpuPwrDomainNode; topology::CORE_COUNT]);
def_static!(PSCI_CPU_DATA: [PsciCpuData; topology::CORE_COUNT]);

static mut REQ_LOCAL_PWR_STATES: [u8; defs::MAX_PWR_LVL as usize * topology::CORE_COUNT] =
    [0; defs::MAX_PWR_LVL as usize * topology::CORE_COUNT];

pub(super) fn non_cpu_pd_lock(idx: usize) -> MCSLockGuard<'static, ()> {
    unsafe { NON_CPU_PD_NODES[idx].lock_var.lock() }
}

pub(super) fn cpu_lock(idx: usize) -> MCSLockGuard<'static, ()> {
    unsafe { CPU_PD_NODES[idx].cpu_lock.lock() }
}

pub(super) fn flush_cache_cpu_state(idx: usize) {
    cache::clean_invalidate(
        unsafe { &PSCI_CPU_DATA[idx].aff_info_state },
        size_of::<AffInfoState>(),
    );
}

pub(super) fn get_cpu_aff_info_state(idx: usize) -> AffInfoState {
    unsafe { read_volatile(&PSCI_CPU_DATA[idx].aff_info_state) }
}

pub(super) fn set_cpu_aff_info_state(idx: usize, state: AffInfoState) {
    unsafe { write_volatile(&mut PSCI_CPU_DATA[idx].aff_info_state, state) }
}

pub(super) fn get_cpu_target_pwrlvl(idx: usize) -> u8 {
    unsafe { read_volatile(&PSCI_CPU_DATA[idx].target_pwrlvl) }
}

pub(super) fn set_cpu_target_pwrlvl(idx: usize, target_pwrlvl: u8) {
    unsafe { write_volatile(&mut PSCI_CPU_DATA[idx].target_pwrlvl, target_pwrlvl) }
}

pub(super) fn get_cpu_local_state(idx: usize) -> u8 {
    unsafe { read_volatile(&PSCI_CPU_DATA[idx].local_state) }
}

pub(super) fn set_cpu_local_state(idx: usize, local_state: u8) {
    unsafe { write_volatile(&mut PSCI_CPU_DATA[idx].local_state, local_state) }
}

pub(super) fn set_non_cpu_pd_cpu_start_idx(idx: usize, cpu_start_idx: usize) {
    unsafe { write_volatile(&mut NON_CPU_PD_NODES[idx].cpu_start_idx, cpu_start_idx) }
}

pub(super) fn get_non_cpu_pd_cpu_start_idx(idx: usize) -> usize {
    unsafe { read_volatile(&mut NON_CPU_PD_NODES[idx].cpu_start_idx) }
}

pub(super) fn set_non_cpu_pd_level(idx: usize, level: u8) {
    unsafe { write_volatile(&mut NON_CPU_PD_NODES[idx].level, level) };
}

pub(super) fn set_non_cpu_pd_parent_node(idx: usize, parent_node: usize) {
    unsafe { write_volatile(&mut NON_CPU_PD_NODES[idx].parent_node, parent_node) };
}

pub(super) fn get_non_cpu_pd_parent_node(idx: usize) -> usize {
    unsafe { read_volatile(&NON_CPU_PD_NODES[idx].parent_node) }
}

pub(super) fn set_non_cpu_pd_local_state(idx: usize, local_state: u8) {
    unsafe { write_volatile(&mut NON_CPU_PD_NODES[idx].local_state, local_state) };
}

pub(super) fn get_non_cpu_pd_local_state(idx: usize) -> u8 {
    unsafe { read_volatile(&NON_CPU_PD_NODES[idx].local_state) }
}

pub(super) fn get_non_cpu_pd_ncpus(idx: usize) -> usize {
    unsafe { read_volatile(&NON_CPU_PD_NODES[idx].ncpus) }
}

pub(super) fn set_non_cpu_pd_ncpus(idx: usize, ncpus: usize) {
    unsafe { write_volatile(&mut NON_CPU_PD_NODES[idx].ncpus, ncpus) }
}

pub(super) fn set_cpu_pd_parent_node(idx: usize, parent_node: usize) {
    unsafe { write_volatile(&mut CPU_PD_NODES[idx].parent_node, parent_node) };
}

pub(super) fn get_cpu_pd_parent_node(idx: usize) -> usize {
    unsafe { read_volatile(&CPU_PD_NODES[idx].parent_node) }
}

pub(super) fn set_cpu_pd_mpidr(idx: usize, mpidr: u64) {
    unsafe { write_volatile(&mut CPU_PD_NODES[idx].mpidr, mpidr) };
}

/// Helper function to update the requested local power state array. This array
/// does not store the requested state for the CPU power level. Hence an
/// assertion is added to prevent us from accessing the CPU power level.
pub(super) fn set_req_local_pwr_state(pwrlvl: usize, core: usize, state: u8) {
    if pwrlvl > PSCI_CPU_PWR_LVL as usize
        && pwrlvl <= defs::MAX_PWR_LVL as usize
        && core < topology::CORE_COUNT
    {
        let idx = topology::CORE_COUNT * (pwrlvl - 1) + core;
        unsafe { write_volatile(&mut REQ_LOCAL_PWR_STATES[idx], state) }
    }
}

/// Helper function to return a reference to an array containing the local power
/// states requested by each cpu for a power domain at 'pwrlvl'. The size of the
/// array will be the number of cpu power domains of which this power domain is
/// an ancestor. These requested states will be used to determine a suitable
/// target state for this power domain during psci state coordination. An
/// assertion is added to prevent us from accessing the CPU power level.
pub(super) fn get_req_local_pwr_states(pwrlvl: usize, cpu_idx: usize) -> &'static [u8] {
    if pwrlvl > PSCI_CPU_PWR_LVL as usize
        && pwrlvl <= defs::MAX_PWR_LVL as usize
        && cpu_idx < topology::CORE_COUNT
    {
        let idx = topology::CORE_COUNT * (pwrlvl - 1) + cpu_idx;
        return unsafe { &REQ_LOCAL_PWR_STATES[idx..] };
    } else {
        panic!("invalid arguments");
    }
}
