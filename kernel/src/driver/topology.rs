#[cfg(any(feature = "raspi3", feature = "raspi4"))]
use super::device::raspi::topology;

#[cfg(feature = "pine64")]
use super::device::allwinner::topology;

pub const MAX_CPUS_PER_CLUSTER: usize = topology::MAX_CPUS_PER_CLUSTER;
pub const CLUSTER_COUNT: usize = topology::CLUSTER_COUNT;
pub const CORE_COUNT: usize = topology::CLUSTER_COUNT * topology::MAX_CPUS_PER_CLUSTER;

pub fn core_pos_by_mpidr(mpidr: usize) -> Option<usize> {
    let core = mpidr & 0xFF;
    let cluster = (mpidr >> 8) & 0xFF;
    let lvl2 = (mpidr >> 16) & 0xFF;
    let lvl3 = (mpidr >> 24) & 0xFF;

    if lvl2 > 0 || lvl3 > 0 || cluster >= CLUSTER_COUNT || core >= MAX_CPUS_PER_CLUSTER {
        None
    } else {
        Some(core)
    }
}
