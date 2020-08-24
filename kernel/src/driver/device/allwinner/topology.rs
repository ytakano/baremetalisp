pub(crate) const MAX_CPUS_PER_CLUSTER: usize = 4;
pub(crate) const CLUSTER_COUNT: usize = 1;
pub(crate) const CORE_COUNT: usize = 4;
pub(crate) const NUM_PWR_DOMAINS: usize = 1 + CLUSTER_COUNT + CORE_COUNT;

pub(crate) const POWER_DOMAIN_TREE_DESC: [u8; 3] = [
    1,                          // One root node for the SoC
    CLUSTER_COUNT as u8,        // One node for each cluster
    MAX_CPUS_PER_CLUSTER as u8, // One set of CPUs per cluster
];
