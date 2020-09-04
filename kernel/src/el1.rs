use crate::aarch64::{cpu, mmu};
use crate::driver::{delays, topology};

#[cfg(not(feature = "raspi3"))]
use crate::aarch64::syscall;

extern "C" {
    fn el0_entry();
}

#[no_mangle]
pub fn el1_entry() -> ! {
    cpu::init_cpacr_el1();

    let addr = mmu::get_memory_map();
    let aff = topology::core_pos() as u64;
    let stack = addr.stack_el0_start - addr.stack_size * aff;
    let entry = el0_entry as *const () as u64;

    // change execution level to EL0t
    cpu::sp_el0::set(stack);
    cpu::spsr_el1::set(0); // EL0t
    cpu::elr_el1::set(entry);
    cpu::eret();

    delays::forever()
}

#[cfg(not(feature = "raspi3"))]
pub fn sys_switch() {
    syscall::smc::to_normal();
}

#[cfg(feature = "raspi3")]
pub fn sys_switch() {
    use crate::driver::uart;
    uart::puts("sys_switch is not supported for Qemu (Raspi3)\n")
}
