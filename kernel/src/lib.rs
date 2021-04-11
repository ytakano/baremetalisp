#![feature(lang_items)]
#![feature(start)]
#![feature(asm)]
#![feature(alloc_error_handler)]
#![no_std]
#![allow(dead_code)]

mod aarch64;
mod bits;
mod boot;
mod driver;
mod el0;
mod el1;

#[macro_use]
extern crate alloc;

use core::panic::PanicInfo;

//-----------------------------------------------------------------------------
// secure world functions

/// entry point from assembly code
#[no_mangle]
pub fn entry() -> ! {
    // Program the counter frequency

    /*
    if aarch64::cpu::get_current_el() == 3 {
        aarch64::cpu::cntfrq_el0::set(driver::defs::SYSCNT_FRQ as u64);
    }

    if aarch64::cpu::get_current_el() == 3 {
        aarch64::cpu::init_cptr_el3(); // enable NEON
    }
    */

    if driver::topology::core_pos() == 0 {
        init_primary();
    } else {
        init_secondary();
    }

    driver::delays::forever()
}

pub fn print_msg(key: &str, val: &str) {
    driver::uart::puts("[");
    driver::uart::puts(key);
    for _ in key.len()..12 {
        driver::uart::puts(" ");
    }
    driver::uart::puts("] ");
    driver::uart::puts(val);
    driver::uart::puts("\n");
}

/// initialization for the primary CPU
fn init_primary() {
    if aarch64::cpu::get_current_el() != 1 {
        panic!("unsupported execution level");
    }

    aarch64::mmu::init_memory_map();
    driver::early_init();

    match aarch64::mmu::init() {
        Some(_) => (),
        None => {
            panic!("failed to initialize MMU");
        }
    };

    driver::init();
    boot::run();
    el1::el1_entry();
}

/// initialization for secondary CPUs
fn init_secondary() -> ! {
    aarch64::cache::invalidate_l1_cache();
    aarch64::cache::invalidate_l2_cache();
    aarch64::cache::invalidate_icache();
    aarch64::mmu::set_regs();

    match aarch64::cpu::get_current_el() {
        1 => driver::delays::forever(),
        _ => panic!("unsupported execution level"),
    }
}

//-----------------------------------------------------------------------------

#[lang = "eh_personality"]
#[no_mangle]
extern "C" fn eh_personality() {}

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

    driver::delays::forever();
}

#[no_mangle]
pub fn abort() -> ! {
    driver::delays::forever();
}

#[no_mangle]
extern "C" fn fmod(x: f64, y: f64) -> f64 {
    libm::fmod(x, y)
}
