use crate::{
    aarch64::mmu,
    cpuint,
    driver::topology,
    process::set_tpid_kernel,
    {allocator, out, paging, process},
};

pub fn kernel_entry() {
    // make tpidrro_el0 kernel space
    set_tpid_kernel();

    // enable IRQ and FIQ
    cpuint::enable_irq();

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
            out::msg("Pager", &msg);

            let msg = format!("0x{:X} - 0x{:X}", s0, e0);
            out::msg("Slab allocator (Kernel)", &msg);

            let msg = format!("0x{:X} - 0x{:X}", s1, e1);
            out::msg("Buddy allocator (Kernel)", &msg);
        }

        // spawn the init process
        process::init();
    }
}
