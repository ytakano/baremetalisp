#![feature(core_intrinsics)]
#![feature(lang_items)]
#![feature(start)]
#![feature(llvm_asm)]
#![feature(alloc_error_handler)]
#![no_std]
#![allow(dead_code)]

mod boot;
mod driver;
mod aarch64;
mod el0;
mod el1;
mod el2;
mod el3;
mod slab;
mod pager;
mod lang;

#[macro_use]
extern crate alloc;

use core::panic::PanicInfo;

#[no_mangle]
fn func() {
    ()
}

#[no_mangle]
pub fn entry() -> ! {
    driver::init();
    let addr =
    match aarch64::mmu::init_firm() {
        Some((a, _)) => a,
        None => {
            driver::uart::puts("Error: failed to initialize MMU\n");
            loop{}
        }
    };

    boot::run();

    match aarch64::el::get_current_el() {
        3 => { el3::el3_to_el1(); }
        2 => {
            driver::uart::puts("Warning: execution level is not EL3\n");
            el2::el2_to_el1(&addr);
        }
        _ => {
            driver::uart::puts("Error: execution level is not EL3\n");
        }
    }

//    driver::uart::puts("halting...\n");
//    driver::power::shutdown();

//    driver::uart::puts("reseting...\n");
//    driver::power::reset();

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