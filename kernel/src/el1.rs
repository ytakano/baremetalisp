use crate::aarch64;
use crate::driver::uart;

extern "C" {
    static mut __stack_el0_end: u64;
    static mut __stack_el0_start: u64;
}

#[no_mangle]
pub fn el1_entry() -> ! {
    uart::puts("Entered EL1\n");
    loop{};

    let end = unsafe { &mut __stack_el0_end as *mut u64 as usize };
    let start = unsafe { &mut __stack_el0_start as *mut u64 as usize };

    let nc = (start - end) >> 21; // div by 2MiB (32 pages), #CPU
    let size = (start - end) / nc;

    let aff = aarch64::cpu::get_affinity_lv0();
    let addr = start - size * aff as usize;

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
            : "r"(addr)
            : "x0"
        );
    }

    loop{}
}