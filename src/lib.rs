#![feature(lang_items)]
#![feature(start)]
#![feature(asm)]
#![no_std]

mod parser;

use core::panic::PanicInfo;

#[no_mangle]
fn func() -> () {
    ()
}

#[no_mangle]
pub fn _start() -> ! {
    unsafe {
        asm!(
            "call func;"
            : : : :
        );
    }

    parser::test(10);

    loop {}
}

#[lang = "eh_personality"] extern fn eh_personality() {}
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub fn abort() -> ! {
    loop {}
}