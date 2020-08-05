use crate::aarch64;

pub fn el2_to_el1() {
    let addr = aarch64::mmu::get_memory_map();
    let aff = aarch64::cpu::get_affinity_lv0();
    let stack = addr.stack_el1_start - addr.stack_size * aff + aarch64::mmu::EL1_ADDR_OFFSET;

    unsafe {
        asm!(
            "mrs {0}, hcr_el2
             orr {0}, {0}, #(1 << 31) // AArch64
             orr {0}, {0}, #(1 << 1)  // SWIO hardwired on Pi3
             msr hcr_el2, {0}

             // enable CNTP for EL1
             mrs {0}, cnthctl_el2
             orr {0}, {0}, #3
             msr cnthctl_el2, {0}
             msr cntvoff_el2, xzr

             // change execution level to EL1
             mov {0}, {1}
             msr sp_el1, {0}    // set stack pointer
             mov {0}, #0b101    // EL1h
             msr spsr_el2, {0}
             adr {0}, el1_entry // set entry point
             msr elr_el2, {0}
             eret",
            out(reg) _,
            in(reg) stack
        );
    }
}
