use crate::aarch64;
use crate::aarch64::context;
use crate::aarch64::cpu;
use crate::driver::topology;
use crate::driver::uart;

extern "C" {
    fn el1_entry();
}

pub fn el3_to_el1() {
    let addr = aarch64::mmu::get_memory_map();
    let aff = topology::core_pos() as u64;
    let stack = addr.stack_el1_start - addr.stack_size * aff + aarch64::mmu::EL1_ADDR_OFFSET;
    let entry = el1_entry as *const () as u64;

    context::set_sp_el1(stack, true); // set stack pointer of EL1
    context::set_elr(entry, true); // set entry point of EL1
    context::restore_and_eret(true); // enter EL1

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

pub fn smc_to_normal(ctx: &context::GpRegs, _sp: usize) {
    if !cpu::is_secure() {
        return;
    }

    // TODO: save SIMD
    context::save_sysregs(true); // save system registers
    context::save_gpregs(ctx, true); // save general purpose registers

    // TODO: restore context

    // TODO: eret

    uart::puts("smc_to_normal is not yet implemented\n");
}

/// secure monitor call
pub fn handle_smc64(ctx: &context::GpRegs) {
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
