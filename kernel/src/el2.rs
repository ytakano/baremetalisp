use crate::aarch64;

extern "C" {
    static mut __stack_el1_end: u64;
    static mut __stack_el1_start: u64;
}

pub fn el2_to_el1() {
    let end = unsafe { &mut __stack_el1_end as *mut u64 as usize };
    let start = unsafe { &mut __stack_el1_start as *mut u64 as usize };

    let nc = (start - end) >> 21; // div by 2MiB (32 pages), #CPU
    let size = (start - end) / nc;

    let aff = aarch64::cpu::get_affinity_lv0();
    let addr = start - size * aff as usize + aarch64::mmu::EL1_ADDR_OFFSET;

    unsafe {
        asm!(
            "mrs x0, hcr_el2
             orr x0, x0, #(1 << 31) // AArch64
             orr x0, x0, #(1 << 1)  // SWIO hardwired on Pi3
             msr hcr_el2, x0

             // enable CNTP for EL1
             mrs x0, cnthctl_el2
             orr x0, x0, #3
             msr cnthctl_el2, x0
             msr cntvoff_el2, xzr

             // change execution level to EL1
             mov x0, $0
             msr sp_el1, x0    // set stack pointer
             mov x0, #0b101    // EL1h
             msr spsr_el2, x0
             adr x0, el1_entry // set entry point
             msr elr_el2, x0
             eret"
            :
            : "r"(addr)
            : "x0"
        );
    }
}