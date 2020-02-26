#![feature(core_intrinsics)]
#![feature(lang_items)]
#![feature(start)]
#![feature(asm)]
#![no_std]

mod parser;
mod boot;
mod driver;
mod aarch64;

use core::panic::PanicInfo;

#[no_mangle]
fn func() {
    ()
}

#[no_mangle]
pub fn entry() -> ! {
    let ctx = driver::init();
    boot::run();

    match ctx.graphics0 {
        Some(mut gr) => {
            driver::uart::puts("drawing mandelbrot set...\n");
            let mut cnt = driver::delays::get_system_timer();
            gr.plot_mandelbrot_set();
            cnt = driver::delays::get_system_timer() - cnt;
            driver::uart::puts("elapsed time to draw: ");
            driver::uart::decimal(cnt);
            driver::uart::puts(" CPU clocks\n");
        }
        None => { driver::uart::puts("failed to initialize graphics\n") }
    }

//    driver::uart::puts("halting...\n");
//    driver::power::shutdown();

//    driver::uart::puts("reseting...\n");
//    driver::power::reset();

    parser::test(10);

    loop {}
}

#[lang = "eh_personality"]
#[no_mangle]
extern fn eh_personality() {}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    driver::uart::puts("kernel panic!");
    loop {}
}

#[no_mangle]
pub fn abort() -> ! {
    loop {}
}