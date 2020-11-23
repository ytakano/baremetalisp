use crate::aarch64::cache;

const CORTEX_A53_ECTLR_SMP_BIT: u64 = 1 << 6;

pub(super) fn core_pwr_down() {
    // Turn off caches.
    cache::disable_dcache_el3();

    // Flush L1 caches.
    cache::flush_l1_cache();

    // Come out of intra cluster coherency
    cache::disable_smp();

    // Turn off instruction cache
    cache::disable_icache_el3();

    // Flush instruction cache
    cache::invalidate_icache();
}

pub(super) fn cluster_pwr_down() {
    // Turn off caches.
    cache::disable_dcache_el3();

    // Flush L1 caches.
    cache::flush_l1_cache();

    // TODO:
    // for brcm stingray
    // Disable the optional ACP.
    // bl      plat_disable_acp

    // Flush L2 caches.
    cache::flush_l2_cache();

    // Come out of intra cluster coherency
    cache::disable_smp();

    // Turn off instruction cache
    cache::disable_icache_el3();

    // Flush instruction cache
    cache::invalidate_icache();
}
