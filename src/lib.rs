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
mod el2;

use core::panic::PanicInfo;

extern "C" {
    static mut __stack_el2_end: u64;
    static mut __stack_el2_start: u64;
}

#[no_mangle]
fn func() {
    ()
}

fn el3_to_el2() {
    let end = unsafe { &mut __stack_el2_end as *mut u64 as usize };
    let start = unsafe { &mut __stack_el2_start as *mut u64 as usize };

    let nc = (start - end) >> 21; // div by 2MiB (32 pages), #CPU
    let size = (start - end) / nc;

    driver::uart::puts("nc = ");
    driver::uart::decimal(nc as u64);
    driver::uart::puts("\n");

    driver::uart::puts("size = ");
    driver::uart::decimal(size as u64);
    driver::uart::puts("\n");

    let aff = aarch64::cpu::get_affinity_lv0();

    let addr = start - size * aff as usize;

    driver::uart::puts("addr = ");
    driver::uart::decimal(addr as u64);
    driver::uart::puts("\n");

    unsafe {
        asm!(
            "mov x0, $0;
             msr sp_el2, x0;    // set stack pointer
             mov x0, #0b1001;   // EL2h
             msr spsr_el3, x0;
             adr x0, el2_entry; // entry point
             msr elr_el3, x0;
             eret"
            :
            : "r"(addr)
            : "x0"
        );
    }
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

    el3_to_el2();

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