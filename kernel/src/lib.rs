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

/// initialization for the master CPU
fn init_master() {
    driver::init();

    let addr =
    match aarch64::mmu::init() {
        Some((a, _, _)) => a,
        None => {
            driver::uart::puts("Error: failed to initialize MMU\n");
            aarch64::delays::infinite_loop();
        }
    };

    boot::run();

//    aarch64::cpu::send_event();
//    aarch64::delays::infinite_loop();

    match aarch64::el::get_current_el() {
        3 => { el3::el3_to_el1(&addr); }
        2 => {
            driver::uart::puts("Warning: execution level is not EL3\n");
            el2::el2_to_el1(&addr);
        }
        _ => {
            driver::uart::puts("Error: execution level is not EL3\n");
        }
    }
}

/// initialization for slave CPUs
fn init_slave() -> ! {
//    aarch64::mmu::set_regs();
    if aarch64::cpu::get_affinity_lv0() == 2 {
        driver::uart::puts("initialized CPU #2\n");
        aarch64::cpu::send_event();
    }
    aarch64::delays::infinite_loop()
}

#[no_mangle]
pub fn entry() -> ! {
    if aarch64::cpu::get_affinity_lv0() == 0 {
        init_master();
    } else {
        aarch64::delays::infinite_loop();
        init_slave();
    }

    aarch64::delays::infinite_loop()
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

    aarch64::delays::infinite_loop();
}

#[no_mangle]
pub fn abort() -> ! {
    aarch64::delays::infinite_loop();
}