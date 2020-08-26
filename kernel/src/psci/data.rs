use core::mem::size_of;
use core::mem::transmute;
use core::ptr::{read_volatile, write_volatile};

use crate::aarch64::{cache, lock};
use crate::driver::topology;

/// The following two data structures implement the power domain tree. The tree
/// is used to track the state of all the nodes i.e. power domain instances
/// described by the platform. The tree consists of nodes that describe CPU power
/// domains i.e. leaf nodes and all other power domains which are parents of a
/// CPU power domain i.e. non-leaf nodes.
pub(crate) struct NonCpuPwrDomainNode {
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
    lock_index: usize,
}

pub(crate) struct CpuPwrDomainNode {
    mpidr: usize,

    // Index of the parent power domain node.
    // TODO: Figure out whether to whether using pointer is more efficient.
    parent_node: usize,

    // A CPU power domain does not require state coordination like its
    // parent power domains. Hence this node does not include a bakery
    // lock. A spinlock is required by the CPU_ON handler to prevent a race
    // when multiple CPUs try to turn ON the same target CPU.
    cpu_lock: lock::LockVar,
}

/// Structure used to store per-cpu information relevant to the PSCI service.
/// It is populated in the per-cpu data array. In return we get a guarantee that
/// this information will not reside on a cache line shared with another cpu.
struct PsciCpuData {
    // State as seen by PSCI Affinity Info API
    aff_info_state: AffInfoState,

    // Highest power level which takes part in a power management
    // operation.
    target_pwrlvl: usize,

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
pub(crate) enum AffInfoState {
    StateOn = 0,
    StateOff,
    StateOnPending,
}

def_static!(CPU_PD_NODES: [CpuPwrDomainNode; topology::CORE_COUNT]);
def_static!(PSCI_CPU_DATA: [PsciCpuData; topology::CORE_COUNT]);
def_static!(PSCI_LOCKS: [lock::LockVar; topology::NUM_NON_CPU_PWR_DOMAINS]);

pub(crate) fn non_pd_lock(idx: usize) -> lock::SpinLock<'static> {
    unsafe { PSCI_LOCKS[idx].lock() }
}

pub(crate) fn cpu_lock(idx: usize) -> lock::SpinLock<'static> {
    unsafe { PSCI_LOCKS[idx].lock() }
}

pub(crate) fn flush_cache_cpu_state(idx: usize) {
    cache::clean_invalidate(
        unsafe { &mut PSCI_CPU_DATA[idx].aff_info_state },
        size_of::<AffInfoState>(),
    );
}

pub(crate) fn get_cpu_state(idx: usize) -> AffInfoState {
    unsafe { read_volatile(&PSCI_CPU_DATA[idx].aff_info_state) }
}

pub(crate) fn set_cpu_state(idx: usize, state: AffInfoState) {
    unsafe { write_volatile(&mut PSCI_CPU_DATA[idx].aff_info_state, state) }
}
