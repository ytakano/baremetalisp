#[cfg(any(feature = "raspi3", feature = "raspi4"))]
use super::device::raspi::topology;

#[cfg(feature = "pine64")]
use super::device::allwinner::topology;

use crate::aarch64::cpu;

pub const MAX_CPUS_PER_CLUSTER: usize = topology::MAX_CPUS_PER_CLUSTER;
pub const CLUSTER_COUNT: usize = topology::CLUSTER_COUNT;
pub const CORE_COUNT: usize = topology::CORE_COUNT;
pub const NUM_PWR_DOMAINS: usize = topology::NUM_PWR_DOMAINS;
pub const NUM_NON_CPU_PWR_DOMAINS: usize = topology::NUM_PWR_DOMAINS - topology::CORE_COUNT;
pub const POWER_DOMAIN_TREE_DESC: &'static [u8] = &topology::POWER_DOMAIN_TREE_DESC;

/// get core index from MPIDR
pub fn core_pos_by_mpidr(mpidr: usize) -> Option<usize> {
    let core = mpidr & 0xFF;
    let cluster = (mpidr >> 8) & 0xFF;
    let lvl2 = (mpidr >> 16) & 0xFF;
    let lvl3 = (mpidr >> 32) & 0xFF;

    if lvl2 > 0 || lvl3 > 0 || cluster >= CLUSTER_COUNT || core >= MAX_CPUS_PER_CLUSTER {
        None
    } else {
        Some(core)
    }
}

/// get my core index
pub fn core_pos() -> usize {
    let mpidr = cpu::mpidr_el1::get();
    core_pos_by_mpidr(mpidr as usize).unwrap()
}
