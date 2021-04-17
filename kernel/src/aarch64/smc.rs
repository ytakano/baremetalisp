use super::mmu;
use crate::driver::topology::core_pos;

extern "C" {
    fn smc_done(arg: u64);
}

pub fn done() {
    // switch to normal world
    let start = mmu::get_stack_el1_start();
    let aff = core_pos() as u64;
    let sp = start - mmu::STACK_SIZE * aff + mmu::EL1_ADDR_OFFSET;

    unsafe {
        asm! {
            "mov     sp, {}
             mov     x0, #0
             b       smc_done",
            in(reg) sp
        }
    }
}
