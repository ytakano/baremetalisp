use crate::aarch64;
use crate::driver::uart;
use crate::driver::{delays, topology};

#[cfg(not(feature = "raspi3"))]
use crate::aarch64::syscall;

#[no_mangle]
pub fn el1_entry() -> ! {
    aarch64::cpu::init_cpacr_el1();

    let addr = aarch64::mmu::get_memory_map();
    let aff = topology::core_pos() as u64;
    let stack = addr.stack_el0_start - addr.stack_size * aff;

    unsafe {
        asm!("
             // change execution level to EL1
             mov {0}, {1}
             msr sp_el0, {0}    // set stack pointer
             mov {0}, #0        // EL0t
             msr spsr_el1, {0}
             adr {0}, el0_entry // set entry point
             msr elr_el1, {0}
             eret",
            out(reg) _,
            in(reg) stack,
        );
    }

    delays::forever()
}

#[cfg(not(feature = "raspi3"))]
pub fn sys_switch() {
    uart::puts("sys_switch\n");
    syscall::smc::to_normal();
}

#[cfg(feature = "raspi3")]
pub fn sys_switch() {
    uart::puts("sys_switch is not supported for Qemu (Raspi3)\n")
}
