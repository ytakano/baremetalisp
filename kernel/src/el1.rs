use crate::aarch64;

#[no_mangle]
pub fn el1_entry() -> ! {
    let addr  = aarch64::mmu::Addr::new();
    let aff   = aarch64::cpu::get_affinity_lv0();
    let stack = addr.stack_el0_start - addr.stack_size * aff;

    unsafe {
        llvm_asm!("
             // change execution level to EL1
             mov x0, $0
             msr sp_el0, x0    // set stack pointer
             mov x0, #0        // EL0t
             msr spsr_el1, x0
             adr x0, el0_entry // set entry point
             msr elr_el1, x0
             eret"
            :
            : "r"(stack)
            : "x0"
        );
    }

    loop{}
}