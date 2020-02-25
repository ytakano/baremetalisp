#![no_std]

use core::panic::PanicInfo;

mod parser;

extern{
    fn uart_puts(s: *const u8);
}

#[no_mangle]
fn func() {
    ()
}

#[no_mangle]
pub fn rust_entry() -> ! {
    unsafe {
        uart_puts("Hello World!\n\0".as_ptr());
    }
    parser::test(10);

    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}