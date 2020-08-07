// This duplicates what the primary cpu did after a cold boot in BL1. The same
// needs to be done when a cpu is hotplugged in. This function could also over-
// ride any EL3 setup done by BL1 as this code resides in rw memory.
pub(crate) fn psci_arch_setup() {}
