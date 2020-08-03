use crate::aarch64;
use crate::aarch64::exception::Context;

pub fn el3_to_el1(addr: &aarch64::mmu::Addr) {
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

             mov {0}, {1}
             msr sp_el1, {0}    // set stack pointer
             mov {0}, #0b101    // EL1h
             msr spsr_el3, {0}
             adr {0}, el1_entry // entry point
             msr elr_el3, {0}
             eret",
            out(reg) _,
            in(reg) stack
        );
    }
}

const FAST_CALL: u32 = 0x80000000;
const SMC64: u32 = 0x40000000;
const MASK_SERVICE: u32 = 0x3f000000;
const MASK_RESERVED: u32 = 0x00ff0000;
const MASK_FUNC: u32 = 0x0000ffff;

/// secure monitor call
pub fn handle_smc64(ctx: &Context) {
    // TODO: save contexts more

    let w0: u32 = ctx.x0 as u32;

    if w0 & SMC64 == 0 {
        return;
    }

    if w0 & FAST_CALL == 0 {
        handle_smc64_yielding();
    } else {
        if w0 & MASK_RESERVED != 0 {
            return;
        }
        handle_smc64_fast();
    }
}

fn handle_smc64_fast() {}

fn handle_smc64_yielding() {}
