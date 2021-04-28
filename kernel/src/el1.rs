use crate::aarch64::mmu;
use crate::driver::topology;
use crate::driver::uart;
use crate::process;

use core::alloc::Layout;
use memalloc::Allocator;

#[global_allocator]
static mut GLOBAL: Allocator = Allocator::new();

extern "C" {
    fn el0_entry();
}

pub fn el1_entry() {
    let aff = topology::core_pos() as u64;

    // spawn init process
    if aff == 0 {
        // initialize memory allocator
        let addr = mmu::get_memory_map();
        let size = addr.el0_heap_end - addr.el0_heap_start;
        let mid = (addr.el0_heap_start + (size >> 1)) as usize;

        unsafe {
            GLOBAL.init_slab(addr.el0_heap_start as usize, (size >> 1) as usize);
            GLOBAL.init_buddy(mid);
        }

        // spawn the init process
        process::init();
    }
}

#[alloc_error_handler]
fn on_oom(layout: Layout) -> ! {
    let size = layout.size() as u64;
    uart::puts("memory allocation error: size = ");
    uart::decimal(size);
    uart::puts("\n");
    loop {}
}
