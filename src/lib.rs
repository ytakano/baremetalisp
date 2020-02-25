#![feature(core_intrinsics)]
#![feature(lang_items)]
#![feature(start)]
#![feature(asm)]
#![no_std]

mod parser;
mod aarch64;

use core::panic::PanicInfo;

#[no_mangle]
fn func() {
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

    let ctx = aarch64::init();

    aarch64::uart::puts("Hello World!\n");
    match aarch64::mbox::get_serial() {
        Some(serial) => {
                aarch64::uart::puts("serial#: ");
                aarch64::uart::hex(serial);
                aarch64::uart::puts("\n");
            }
        None => { aarch64::uart::puts("failed to get serial#\n") }
    }

    aarch64::uart::puts("random number:\n");
    for _ in 0..5 {
        let rnd = aarch64::rand::rand64();
        aarch64::uart::hex(rnd);
        aarch64::uart::puts("\n");
    }

    aarch64::uart::puts("system timer's counter: ");
    let cnt = aarch64::delays::get_system_timer();
    aarch64::uart::hex(cnt);
    aarch64::uart::puts("\n");

    match ctx.graphics0 {
        Some(mut gr) => {
            aarch64::uart::puts("graphics was successfuly initialized\n");
            gr.plot_mandelbrot_set();
        }
        None => { aarch64::uart::puts("failed to initialize graphics\n") }
    }

//    aarch64::uart::puts("halting...\n");
//    aarch64::power::shutdown();

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