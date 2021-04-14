use crate::aarch64::{cpu, mmu};
use crate::driver::topology;
use crate::driver::uart;
use crate::thread;

use core::alloc::Layout;
use memalloc::Allocator;

#[global_allocator]
static mut GLOBAL: Allocator = Allocator::new();

extern "C" {
    fn el0_entry_core_0();
    fn el0_entry_core_x();
}

#[no_mangle]
pub fn el1_entry() {
    // initialize memory allocator
    let addr = mmu::get_memory_map();
    let size = addr.el0_heap_end - addr.el0_heap_start;
    let mid = (addr.el0_heap_start + (size >> 1)) as usize;
    unsafe {
        GLOBAL.init_slab(addr.el0_heap_start as usize, (size >> 1) as usize);
        GLOBAL.init_buddy(mid);
    }

    let addr = mmu::get_memory_map();
    let aff = topology::core_pos() as u64;
    let stack = addr.stack_el0_start - addr.stack_size * aff;
    let entry = if aff == 0 {
        el0_entry_core_0
    } else {
        el0_entry_core_x
    } as *const () as u64;

    if aff == 0 {
        if let Some(_id) = thread::spawn() {
        } else {
            panic!("failed to spawn thread");
        }
    }

    // change execution level to EL0t
    cpu::sp_el0::set(stack);
    cpu::spsr_el1::set(0); // EL0t
    cpu::elr_el1::set(entry);
    cpu::eret();

    return;
}

#[alloc_error_handler]
fn on_oom(layout: Layout) -> ! {
    let size = layout.size() as u64;
    uart::puts("memory allocation error: size = ");
    uart::decimal(size);
    uart::puts("\n");
    loop {}
}
