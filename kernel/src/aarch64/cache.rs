use super::{cpu, mmu};
use core::arch::asm;

pub fn flush() {
    let mut start = mmu::get_data_start();
    let end = mmu::get_memory_map().pager_mem_end;

    cpu::dmb_sy();
    while start < end {
        unsafe { asm!("dc civac, {}", in(reg) start) };
        start += mmu::PAGESIZE;
    }

    cpu::dmb_sy();
}
