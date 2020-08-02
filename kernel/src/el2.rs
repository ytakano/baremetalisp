use crate::aarch64;

pub fn el2_to_el1(addr: &aarch64::mmu::Addr) {
    let aff = aarch64::cpu::get_affinity_lv0();
    let stack = addr.stack_el1_start - addr.stack_size * aff + aarch64::mmu::EL1_ADDR_OFFSET;

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
             mov x0, {}
             msr sp_el1, x0    // set stack pointer
             mov x0, #0b101    // EL1h
             msr spsr_el2, x0
             adr x0, el1_entry // set entry point
             msr elr_el2, x0
             eret",
            in(reg) stack
        );
    }
}
