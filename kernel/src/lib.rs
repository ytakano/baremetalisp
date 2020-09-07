#![feature(lang_items)]
#![feature(start)]
#![feature(llvm_asm)]
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
mod el2;
mod el3;
mod pager;
mod psci;
mod slab;

#[macro_use]
extern crate alloc;

use core::panic::PanicInfo;

//-----------------------------------------------------------------------------
// secure world functions

/// entry point from assembly code
#[no_mangle]
pub fn entry() -> ! {
    // Program the counter frequency
    if aarch64::cpu::get_current_el() == 3 {
        aarch64::cpu::cntfrq_el0::set(driver::defs::SYSCNT_FRQ as u64);
    }

    if aarch64::cpu::get_current_el() == 3 {
        aarch64::cpu::init_cptr_el3(); // enable NEON
    }

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
    aarch64::mmu::init_memory_map();

    driver::early_init();

    match aarch64::mmu::init() {
        Some(_) => (),
        None => {
            panic!("failed to initialize MMU");
        }
    };
    driver::init();

    // examples
    // driver::psci::pwr_domain_on(1); // wake up CPU #1 (Pine64)
    // driver::delays::wait_milisec(10);

    // aarch64::cpu::start_non_primary(); // wake up non-primary CPUs (Raspi)

    match aarch64::cpu::get_current_el() {
        3 => {
            psci::init();
            aarch64::context::init_secure();
            aarch64::context::init_el2_regs();
            print_msg("PSCI", "enabled");
            boot::run();
            el3::el3_to_el1();
        }
        2 => {
            print_msg("Warning", "execution level is not EL3");
            print_msg("PSCI", "disabled");
            boot::run();
            aarch64::context::init_el2_regs();
            el2::el2_to_el1();
        }
        _ => {
            panic!("execution level is not EL3");
        }
    }
}

/// initialization for secondary CPUs
fn init_secondary() -> ! {
    aarch64::mmu::set_regs();

    match aarch64::cpu::get_current_el() {
        3 => {
            psci::init_warmboot();
            aarch64::context::init_secure();
            aarch64::context::init_el2_regs();

            driver::uart::puts("booted and initialized CPU #");
            driver::uart::decimal(driver::topology::core_pos() as u64);
            driver::uart::puts("\n");

            el3::el3_to_el1();
        }
        2 => {
            aarch64::context::init_el2_regs();
        }
        _ => {
            panic!("execution level is not EL3");
        }
    }

    driver::delays::forever()
}

//-----------------------------------------------------------------------------
// normal world functions
#[no_mangle]
pub fn ns_entry() -> ! {
    unsafe {
        asm!(
            "ldr x1, =__ram_start
             mov x2, #1024 * 1024 * 256
             mrs x3, mpidr_el1 // read cpu id
             and x3, x3, #0xFF
             mov x4, #1024 * 1024 * 2
             mul x3, x3, x4
             add x2, x2, x3
             add x1, x1, x2
             mov sp, x1"
        );
    }
    non_secure()
}

pub fn non_secure() -> ! {
    driver::uart::puts("Hello Normal World from CPU #");
    driver::uart::decimal(driver::topology::core_pos() as u64);
    driver::uart::puts("\n");
    /*
        // test code for CPU on
        if driver::topology::core_pos() == 0 {
            // wake CPU #1 up
            driver::uart::puts("waking CPU #1 up\n");
            unsafe {
                let x0: u64 = psci::PSCI_CPU_ON_AARCH64 as u64;
                asm!(
                    "mov x0, {}
                     mov x1, 1 // CPU #1
                     adr x2, ns_entry // set entry point
                     mov x3, xzr
                     smc #0",
                    in(reg) x0,
                );
            }
        } else {
            driver::delays::forever();
        }
    */

    /*
    // test code for shutdown
    unsafe {
        let x0: u64 = psci::PSCI_SYSTEM_RESET as u64;
        asm!(
            "mov x0, {}
             smc #0",
            in(reg) x0
        );
    }*/

    driver::delays::wait_milisec(200);

    loop {
        aarch64::syscall::smc::to_secure();
        driver::uart::puts("Hello Normal World from CPU #");
        driver::uart::decimal(driver::topology::core_pos() as u64);
        driver::uart::puts("\n");
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
