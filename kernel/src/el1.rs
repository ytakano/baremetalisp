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
        paging::init(addr.pager_mem_start as usize, addr.pager_mem_end as usize);

        // initialize Kernel heap
        let (s0, e0, s1, e1) = allocator::init_kernel();

        {
            let msg = format!("0x{:X} - 0x{:X}", addr.pager_mem_start, addr.pager_mem_end);
            print::msg("Pager", &msg);

            let msg = format!("0x{:X} - 0x{:X}", s0, e0);
            print::msg("Slab allocator (Kernel)", &msg);

            let msg = format!("0x{:X} - 0x{:X}", s1, e1);
            print::msg("Buddy allocator (Kernel)", &msg);
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
