use crate::driver;
use crate::boot;

use alloc::vec::Vec;

#[no_mangle]
pub fn el1_entry() -> ! {
    driver::uart::puts("entered EL1\n");
    boot::run();

    let mut xs = Vec::new();
    xs.push(42);

    let p = 0x400000000 as *mut u64;
    unsafe { *p = 10 };

    loop{}
}