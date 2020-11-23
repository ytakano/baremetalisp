pub(in crate::driver) const MAX_CPUS_PER_CLUSTER: usize = 4;
pub(in crate::driver) const CLUSTER_COUNT: usize = 1;
pub(in crate::driver) const CORE_COUNT: usize = 4;
pub(in crate::driver) const NUM_PWR_DOMAINS: usize = CLUSTER_COUNT + CORE_COUNT;

pub(in crate::driver) const POWER_DOMAIN_TREE_DESC: [u8; 2] = [
    CLUSTER_COUNT as u8,        // Number of root nodes
    MAX_CPUS_PER_CLUSTER as u8, // Number of children for the first node
];
