#![feature(core_intrinsics)]
#![feature(lang_items)]
#![feature(start)]
#![feature(asm)]
#![no_std]

mod parser;
mod aarch64;

use core::panic::PanicInfo;

#[no_mangle]
fn func() -> () {
    ()
}

#[no_mangle]
pub fn entry() -> ! {
    unsafe {
        asm!(
            "bl func;"
            : : : :
        );
    }

    aarch64::init();

    aarch64::uart::puts("Hello World!\n");
    match aarch64::mbox::get_serial() {
        Some(_) => {
                aarch64::uart::puts("serial#: ");
                aarch64::uart::puts("\n");
            }
        None => { aarch64::uart::puts("failed to get serial#\n") }
    };

    parser::test(10);

    loop {}
}

#[lang = "eh_personality"]
#[no_mangle]
extern fn eh_personality() {}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub fn abort() -> ! {
    loop {}
}