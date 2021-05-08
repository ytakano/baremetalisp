use crate::{
    aarch64::{cpu, mmu},
    driver::{topology, uart},
    process::set_tpid_kernel,
    {allocator, paging, print, process},
};

use core::alloc::Layout;

extern "C" {
    fn el0_entry();
}

pub fn el1_entry() {
    // make tpidrro_el0 kernel space
    set_tpid_kernel();

    // enable IRQ and FIQ
    let daif = cpu::daif::get();
    cpu::daif::set(daif & !((cpu::DAIF_IRQ_BIT | cpu::DAIF_FIQ_BIT) << cpu::SPSR_DAIF_SHIFT));

    let aff = topology::core_pos() as u64;

    // spawn init process
    if aff == 0 {
        let addr = mmu::get_memory_map();

        // initialize Pager
        paging::init(addr.el0_heap_start as usize, addr.el0_heap_end as usize);

        // initialize Kernel heap
        allocator::init_kernel();
        print::msg("Kernel heap", "initialized");

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
