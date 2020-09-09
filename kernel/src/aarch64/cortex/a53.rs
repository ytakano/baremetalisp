use crate::aarch64::cpu;

const CORTEX_A53_ECTLR_SMP_BIT: u64 = 1 << 6;

extern "C" {
    fn dcsw_op_level1(x0: u64);
}

pub(crate) fn core_pwr_down() {
    // Turn off caches.
    disable_dcache();

    // Flush L1 caches.
    unsafe { dcsw_op_level1(cpu::DCCISW) };

    // Come out of intra cluster coherency
    disable_smp();
}

/// Disable L1 data cache and unified L2 cache
fn disable_dcache() {
    let sctlr_el3 = cpu::sctlr_el3::get();
    cpu::sctlr_el3::set(sctlr_el3 & !cpu::SCTLR_C_BIT);
    cpu::isb();
}

/// Disable intra-cluster coherency
fn disable_smp() {
    unsafe {
        asm!("mrs {0}, S3_1_C15_C2_0
              bic {0}, {0}, {1}
              msr S3_1_C15_C2_0, {0}",
              out(reg) _,
              in(reg) CORTEX_A53_ECTLR_SMP_BIT);
    }
    cpu::isb();
    cpu::dmb_sy();
}
