use crate::aarch64::syscall::smc;
use crate::aarch64::{context, cpu, mmu};
use crate::driver::delays;
use crate::driver::topology;
use crate::psci;

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

/// switch between secure and normal world
/// if to_secure is true then switch to secure world
/// otherwise switch to normal world
pub fn switch_world(ctx: &context::GpRegs, sp: usize, to_secure: bool) {
    if cpu::is_secure() == to_secure {
        return;
    }

    let c = context::get_ctx(topology::core_pos(), !to_secure);
    c.save_fpregs(); // save SIMD registers
    c.save_sysregs(); // save system registers
    c.save_gpregs(ctx); // save general purpose registers

    context::restore_and_eret(sp as u64, to_secure);
}

/// SMC standard service calls
pub fn smc_std_service(ctx: &context::GpRegs, sp: usize) {
    match ctx.x0 {
        smc::SMC_TO_NORMAL => switch_world(ctx, sp, false),
        smc::SMC_TO_SECURE => switch_world(ctx, sp, true),
        _ => {
            if psci::is_psci_fid(ctx.x0 as u32) {
                psci::smc_handler(
                    ctx.x0 as u32,
                    ctx.x1 as usize,
                    ctx.x2 as usize,
                    ctx.x3 as usize,
                );
            }
        }
    }
}
