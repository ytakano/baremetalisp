#![feature(rustc_private)]
#![feature(lang_items)]
#![feature(start)]
#![no_std]

extern crate libc;

mod parser;

use core::panic::PanicInfo;

// Entry point for this program
#[no_mangle]
#[start]
fn start(_argc: isize, _argv: *const *const u8) -> isize {
    parser::test(10);
    0
}

#[lang = "eh_personality"] extern fn eh_personality() {}
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}