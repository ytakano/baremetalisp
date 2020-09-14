use super::cpu;
use super::mmu;
use super::mmu::PAGESIZE;

const LEVEL_SHIFT: u64 = 1;

extern "C" {
    fn dcsw_op_level1(x0: u64);
    fn dcsw_op_level2(x0: u64);
}

pub fn invalidate_l1_cache() {
    unsafe { dcsw_op_level1(cpu::DCISW) };
}

pub fn invalidate_l2_cache() {
    unsafe { dcsw_op_level2(cpu::DCISW) };
}

pub fn flush_l1_cache() {
    unsafe { dcsw_op_level1(cpu::DCCISW) };
}

pub fn flush_l2_cache() {
    unsafe { dcsw_op_level2(cpu::DCCISW) };
}

/// Enable intra-cluster coherency
pub fn enable_smp() {
    let flag: u64 = cpu::ECTLR_SMP_BIT;
    unsafe {
        asm!("mrs {0}, S3_1_C15_C2_0
              orr {0}, {0}, {1}
              msr S3_1_C15_C2_0, {0}",
              out(reg) _,
              in(reg) flag);
    }
    cpu::isb();
    cpu::dmb_sy();
}

/// Disable intra-cluster coherency
pub fn disable_smp() {
    let flag: u64 = cpu::ECTLR_SMP_BIT;
    unsafe {
        asm!("mrs {0}, S3_1_C15_C2_0
              bic {0}, {0}, {1}
              msr S3_1_C15_C2_0, {0}",
              out(reg) _,
              in(reg) flag);
    }
    cpu::isb();
    cpu::dmb_sy();
}

pub fn invalidate_icache() {
    unsafe { asm!("ic iallu") };
}

/// Disable L1 data cache and unified L2 cache
pub fn disable_dcache_el3() {
    let sctlr_el3 = cpu::sctlr_el3::get();
    cpu::sctlr_el3::set(sctlr_el3 & !cpu::SCTLR_C_BIT);
    cpu::isb();
}

/// Disable instruction cache
pub fn disable_icache_el3() {
    let sctlr_el3 = cpu::sctlr_el3::get();
    cpu::sctlr_el3::set(sctlr_el3 & !cpu::SCTLR_I_BIT);
    cpu::isb();
}

/// clean cache.
/// dc cvac
pub fn clean<T>(obj: &T, size: usize) {
    let addr = obj as *const T as usize;
    let mut base = addr & !(PAGESIZE as usize - 1);

    cpu::dmb_sy();
    while base < addr + size {
        unsafe { asm!("dc cvac, {}", in(reg) base) };
        base += PAGESIZE as usize;
    }

    cpu::dmb_sy();
}

/// flush cache
/// dc civac
pub fn clean_invalidate<T>(obj: &T, size: usize) {
    let addr = obj as *const T as usize;
    let mut base = addr & !(PAGESIZE as usize - 1);

    cpu::dmb_sy();
    while base < addr + size {
        unsafe { asm!("dc civac, {}", in(reg) base) };
        base += PAGESIZE as usize;
    }

    cpu::dmb_sy();
}

/// invalidate cache
/// dc ivac
pub fn invalidate<T>(obj: &T, size: usize) {
    let addr = obj as *const T as usize;
    let mut base = addr & !(PAGESIZE as usize - 1);

    cpu::dmb_sy();
    while base < addr + size {
        unsafe { asm!("dc ivac, {}", in(reg) base) };
        base += PAGESIZE as usize;
    }

    cpu::dmb_sy();
}

pub fn invalidate_global() {
    let mut start = mmu::get_data_start();
    let end = mmu::get_bss_end();

    cpu::dmb_sy();
    while start < end {
        unsafe { asm!("dc ivac, {}", in(reg) start) };
        start += PAGESIZE;
    }

    cpu::dmb_sy();
}
