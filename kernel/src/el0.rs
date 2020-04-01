use crate::slab;
use crate::parser;

use alloc::string::ToString;

#[no_mangle]
pub fn el0_entry() -> ! {
    // initialize memory allocator
    slab::init();

    parser::parse_expr(&"(add 1 2)".to_string());

    let p = 0x400000000 as *mut u64;
    unsafe { *p = 10 };

    loop{}
}