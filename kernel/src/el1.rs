use crate::aarch64::{cpu, mmu};
use crate::driver::topology;
use crate::driver::uart;
use crate::{allocator, paging, print, process};

use core::alloc::Layout;
use memalloc::Allocator;

const BUDDY_SIZE: usize = 1024 * 1024 * 32;

/// will be deprecated
#[global_allocator]
static mut GLOBAL: Allocator = Allocator::new();

extern "C" {
    fn el0_entry();
}

pub fn el1_entry() {
    // enable IRQ and FIQ
    let daif = cpu::daif::get();
    cpu::daif::set(daif & !((cpu::DAIF_IRQ_BIT | cpu::DAIF_FIQ_BIT) << cpu::SPSR_DAIF_SHIFT));

    let aff = topology::core_pos() as u64;

    // spawn init process
    if aff == 0 {
        // initialize memory allocator
        // will be deprecated
        init_allocator();

        let addr = mmu::get_memory_map();

        // initialize Pager
        paging::init(addr.el0_heap_start as usize, addr.el0_heap_end as usize);

        // initialize Kernel heap
        allocator::init_kern();

        print::msg("Kernel heap", "initialized");

        // testing
        let mut a = alloc::boxed::Box::new(10);
        *a = 200;

        print::decimal("*a", *a);

        let ptr = alloc::boxed::Box::into_raw(a);
        print::hex64("ptr", ptr as u64);

        unsafe { alloc::boxed::Box::from_raw(ptr) };

        // spawn the init process
        process::init();
    }
}

/// will be deprecated
fn init_allocator() {
    // initialize memory allocator
    let addr = mmu::get_memory_map();

    let slab_size = (addr.el0_heap_end - addr.el0_heap_start) as usize - BUDDY_SIZE;
    let slab_start = addr.el0_heap_start as usize + BUDDY_SIZE;

    unsafe {
        GLOBAL.init_buddy(addr.el0_heap_start as usize);
        GLOBAL.init_slab(slab_start, slab_size);
    }

    let msg = format!("0x{:x} - 0x{:x} (32MiB)", addr.el0_heap_start, slab_start);
    print::msg("Buddy Alloc", &msg);

    let msg = format!(
        "0x{:x} - 0x{:x} ({}MiB)",
        slab_start,
        slab_start + slab_size,
        slab_size >> 20
    );
    print::msg("Slab Alloc", &msg);
}

#[alloc_error_handler]
fn on_oom(layout: Layout) -> ! {
    let size = layout.size() as u64;
    uart::puts("memory allocation error: size = ");
    uart::decimal(size);
    uart::puts("\n");
    loop {}
}
