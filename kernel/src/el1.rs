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

    let mut list = LinkedList::new();

    for _ in 0..8 {
        for i in 0..128 {
            list.push_back([i as u64; 23]);
        }
        driver::uart::puts("---------------------------------------\n");
        slab::print_slabs();
    }

    let p = 0x400000000 as *mut u64;
    unsafe { *p = 10 };

    loop{}
}