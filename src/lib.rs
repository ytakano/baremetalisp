#![feature(core_intrinsics)]
#![feature(lang_items)]
#![feature(start)]
#![feature(asm)]
#![no_std]
#![allow(dead_code)]

mod parser;
mod boot;
mod driver;
mod aarch64;

use core::panic::PanicInfo;

#[no_mangle]
fn func() {
    ()
}

extern "C" {
    fn init_mmu() -> ();
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

    let p = 0x400000000 as *mut u64;
    unsafe { *p = 10 };

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
fn panic(info: &PanicInfo) -> ! {
    driver::uart::puts("kernel panic!\n");
    if let Some(location) = info.location() {
        driver::uart::puts(location.file());
        driver::uart::puts(":");
        driver::uart::decimal(location.line() as u64);
        driver::uart::puts("\n");
    }

    if let Some(s) = info.payload().downcast_ref::<&str>() {
        driver::uart::puts(s);
        driver::uart::puts("\n");
    }

    loop {}
}

#[no_mangle]
pub fn abort() -> ! {
    loop {}
}