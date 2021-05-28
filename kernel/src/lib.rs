#![feature(lang_items)]
#![feature(start)]
#![feature(asm)]
#![feature(alloc_error_handler)]
#![no_std]
#![allow(dead_code)]

mod aarch64;
mod allocator;
mod bsp;
mod cpuint;
mod driver;
mod global;
mod kernel;
mod mmio;
mod out;
mod paging;
mod process;
mod smc;
mod splash;
mod syscall;
mod tty;
mod userland;

#[macro_use]
extern crate alloc;

use core::panic::PanicInfo;

//-----------------------------------------------------------------------------
// secure world functions

/// entry point from assembly code
#[no_mangle]
pub fn entry() {
    if driver::topology::core_pos() == 0 {
        init_primary();
    } else {
        // called from vector_cpu_on_entry
        init_secondary();
        return;
    }

    bsp::delays::forever()
}

/// initialization for the primary CPU
fn init_primary() {
    bsp::early_init();
    driver::early_init(); // TODO: remove this line

    if aarch64::cpu::get_current_el() != 1 {
        panic!("unsupported execution level");
    }

    aarch64::mmu::init_memory_map();

    match aarch64::mmu::init() {
        Some(_) => init_primary2(),
        None => {
            panic!("failed to initialize MMU");
        }
    };
}

#[inline(never)]
fn init_primary2() {
    bsp::init();
    driver::init();
    splash::run();
    kernel::kernel_entry();
}

/// initialization for secondary CPUs
fn init_secondary() {
    match aarch64::cpu::get_current_el() {
        1 => (),
        _ => panic!("unsupported execution level"),
    }

    aarch64::mmu::set_regs();
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

    bsp::delays::forever();
}

#[no_mangle]
pub fn abort() -> ! {
    bsp::delays::forever();
}

#[no_mangle]
extern "C" fn fmod(x: f64, y: f64) -> f64 {
    libm::fmod(x, y)
}
