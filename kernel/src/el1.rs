use crate::driver;
use crate::boot;
use crate::slab;

//use alloc::vec::Vec;
use alloc::collections::linked_list::LinkedList;

#[no_mangle]
pub fn el1_entry() -> ! {
    driver::uart::puts("entered EL1\n");
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