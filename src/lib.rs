#![no_std]

use core::panic::PanicInfo;

mod raspi;
mod parser;

#[no_mangle]
fn func() {
    ()
}

#[no_mangle]
pub fn rust_entry() -> ! {
    raspi::uart::puts("Hello World!\n");

    let serial = raspi::mbox::get_serial();
    raspi::uart::puts("serial#: ");
    raspi::uart::hex(serial);
    raspi::uart::puts("\n");

    match raspi::graphics::init() {
        Some(mut pict) => {
            pict.draw_mandelbrot_set();
        }
        _ => {
            raspi::uart::puts("failed to initialize graphics\n");
        }
    }

    parser::test(10);

    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    raspi::uart::puts("kernel panic!\n");
    loop {}
}