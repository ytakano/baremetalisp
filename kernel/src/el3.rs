use crate::aarch64::{context, cpu, mmu};
use crate::driver::delays;
use crate::driver::topology;
use crate::driver::uart;

extern "C" {
    fn el1_entry();
    static __stack_firm_start: u64;
}

pub fn el3_to_el1() -> ! {
    let addr = mmu::get_memory_map();
    let aff = topology::core_pos() as u64;
    let stack = addr.stack_el1_start - addr.stack_size * aff + mmu::EL1_ADDR_OFFSET;
    let entry = el1_entry as *const () as u64;
    let stack_el3 = mmu::get_stack_firm_start() - addr.stack_size * aff;

    context::set_sp_el1(stack, true); // set stack pointer of EL1
    context::set_elr(entry, true); // set entry point of EL1
    context::restore_and_eret(stack_el3, true); // enter EL1

    delays::forever();
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
