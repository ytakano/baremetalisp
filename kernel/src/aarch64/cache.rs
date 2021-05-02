use super::cpu;
use super::mmu;

pub fn flush() {
    let mut start = mmu::get_data_start();
    let end = mmu::get_memory_map().el0_heap_end;

    cpu::dmb_sy();
    while start < end {
        unsafe { asm!("dc civac, {}", in(reg) start) };
        start += mmu::PAGESIZE;
    }

    cpu::dmb_sy();
}
