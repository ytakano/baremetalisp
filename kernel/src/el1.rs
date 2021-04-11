use crate::aarch64::{cpu, mmu};
use crate::driver::{delays, topology};

extern "C" {
    fn el0_entry_core_0();
    fn el0_entry_core_x();
}

#[no_mangle]
pub fn el1_entry() -> ! {
    let addr = mmu::get_memory_map();
    let aff = topology::core_pos() as u64;
    let stack = addr.stack_el0_start - addr.stack_size * aff;
    let entry = if topology::core_pos() == 0 {
        el0_entry_core_0
    } else {
        el0_entry_core_x
    } as *const () as u64;

    // change execution level to EL0t
    cpu::sp_el0::set(stack);
    cpu::spsr_el1::set(0); // EL0t
    cpu::elr_el1::set(entry);
    cpu::eret();

    delays::forever()
}
