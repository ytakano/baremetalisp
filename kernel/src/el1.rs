use crate::aarch64;
use crate::driver::delays;

#[no_mangle]
pub fn el1_entry() -> ! {
    let addr = aarch64::mmu::get_memory_map();
    let aff = aarch64::cpu::get_affinity_lv0();
    let stack = addr.stack_el0_start - addr.stack_size * aff;

    unsafe {
        asm!("
             // change execution level to EL1
             mov {0}, {1}
             msr sp_el0, {0}    // set stack pointer
             mov {0}, #0        // EL0t
             msr spsr_el1, {0}
             adr {0}, el0_entry // set entry point
             msr elr_el1, {0}
             eret",
            out(reg) _,
            in(reg) stack,
        );
    }

    delays::forever()
}
