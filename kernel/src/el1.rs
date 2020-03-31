use crate::aarch64;
use crate::driver;
use crate::boot;
use crate::slab;

extern "C" {
    static mut __stack_el0_end: u64;
    static mut __stack_el0_start: u64;
}

//use alloc::vec::Vec;
use alloc::collections::linked_list::LinkedList;

#[no_mangle]
pub fn el1_entry() -> ! {
    driver::uart::puts("entered EL1\n");

    let end = unsafe { &mut __stack_el0_end as *mut u64 as usize };
    let start = unsafe { &mut __stack_el0_start as *mut u64 as usize };

    let nc = (start - end) >> 21; // div by 2MiB (32 pages), #CPU
    let size = (start - end) / nc;

    let aff = aarch64::cpu::get_affinity_lv0();
    let addr = start - size * aff as usize;

    driver::uart::puts("addr = 0x");
    driver::uart::hex(start as u64);
    driver::uart::puts("\n");

    unsafe {
        asm!("
             // change execution level to EL1
             mov x0, $0
             msr sp_el0, x0    // set stack pointer
             mov x0, #0        // EL0t
             msr spsr_el1, x0
             adr x0, el0_entry // set entry point
             msr elr_el1, x0
             eret"
            :
            : "r"(addr)
            : "x0"
        );
    }
    boot::run();

    slab::init();
    {
        let mut list = LinkedList::new();

        for _ in 0..8 {
            driver::uart::puts("PUSH:\n");
            for i in 0..911 {
                list.push_back([i as u64; 4]);
            }
            slab::print_slabs();
            driver::uart::puts("---------------------------------------\n");

            driver::uart::puts("POP:\n");
            for _ in 0..419 {
                list.pop_front();
            }
            slab::print_slabs();
            driver::uart::puts("---------------------------------------\n");
        }
    }

    let p = 0x400000000 as *mut u64;
    unsafe { *p = 10 };

    loop{}
}