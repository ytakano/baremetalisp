use core::slice;

use super::el;
use crate::driver;

extern "C" {
    static mut __bss_start: u64;

    static mut __stack_el3_end: u64;
    static mut __stack_el3_start: u64;
    static mut __stack_el2_end: u64;
    static mut __stack_el2_start: u64;

    static mut __tt_el3_end: u64;
    static mut __tt_el3_start: u64;
    static mut __tt_el2_end: u64;
    static mut __tt_el2_start: u64;

    static mut _end: u64;
}

pub struct VMTables {
    el2: &'static mut [u64],
    el3: &'static mut [u64]
}

pub fn enabled() -> Option<bool> {
    let mut sctlr: u32;

    let el = el::get_current_el();
    if el == 1 {
        unsafe { asm!("mrs $0, SCTLR_EL1" : "=r"(sctlr)) };
        Some(sctlr & 1 == 1)
    } else if el == 2 {
        unsafe { asm!("mrs $0, SCTLR_EL2" : "=r"(sctlr)) };
        Some(sctlr & 1 == 1)
    } else if el == 3 {
        unsafe { asm!("mrs $0, SCTLR_EL3" : "=r"(sctlr)) };
        Some((sctlr & 1) == 1)
    } else {
        None
    }
}

// 64KB page
// level 2 and 3 translation tables

const PAGESIZE: u64 = 64 * 1024;

// NSTable (63bit)
const FLAG_L2_NS: u64 = 1 << 63; // non secure table


const FLAG_L3_XN:   u64 = 1 << 54; // execute never
const FLAG_L3_PXN:  u64 = 1 << 53; // priviledged execute
const FLAG_L3_CONT: u64 = 1 << 52; // contiguous
const FLAG_L3_DBM:  u64 = 1 << 51; // dirty bit modifier
const FLAG_L3_AF:   u64 = 1 << 10; // access flag
const FLAG_L3_NS:   u64 = 1 <<  5; // non secure

// [9:8]: Shareability attribute, for Normal memory
//    | Shareability
// ---|------------------
// 00 | non sharedable
// 01 | reserved
// 10 | outer sharedable
// 11 | inner sharedable
const FLAG_L3_OSH: u64 = 0b10 << 8;
const FLAG_L3_ISH: u64 = 0b11 << 8;

// [7:6]: access permissions
//    | Access from            |
//    | higher Exception level | Access from EL0
// ---|------------------------|-----------------
// 00 | read/write             | none
// 01 | read/write             | read/write
// 10 | read-only              | none
// 11 | read-only              | read-only
const FLAG_L3_SH_RW_N:  u64 =    0;
const FLAG_L3_SH_RW_RW: u64 =    1 << 6;
const FLAG_L3_SH_R_N:   u64 = 0b10 << 6;
const FLAG_L3_SH_R_R:   u64 = 0b11 << 6;

// [4:2]: AttrIndx
// defined in MAIR register
const FLAG_L3_ATTR_MEM: u64 = 0     ; // normal memory
const FLAG_L3_ATTR_DEV: u64 = 1 << 2; // device MMIO
const FLAG_L3_ATTR_NC:  u64 = 2 << 2; // non-cachable


#[cfg(any(feature = "raspi3", feature = "raspi2"))]
pub const DRIVER_MEM_START: usize =  0x3C000000;

#[cfg(any(feature = "raspi3", feature = "raspi2"))]
pub const DRIVER_MEM_END:   usize =  0x40000000;

#[cfg(feature = "raspi4")]
pub const DRIVER_MEM_START: usize =  0xfd000000; // maybe...

#[cfg(feature = "raspi4")]
pub const DRIVER_MEM_END:   usize = 0x100100000; // maybe...

pub fn print_addr() {
    let addr = unsafe { &mut __bss_start as *mut u64 as u64 };
    driver::uart::puts("__bss_start       = ");
    driver::uart::decimal(addr as u64);
    driver::uart::puts("\n");

    let addr = unsafe { &mut __tt_el3_start as *mut u64 as u64 };
    driver::uart::puts("__tt_el3_start    = ");
    driver::uart::decimal(addr as u64);
    driver::uart::puts("\n");

    let addr = unsafe { &mut __tt_el2_start as *mut u64 as u64 };
    driver::uart::puts("__tt_el2_start    = ");
    driver::uart::decimal(addr as u64);
    driver::uart::puts("\n");

    let addr = unsafe { &mut __stack_el3_start as *mut u64 as u64 };
    driver::uart::puts("__stack_el3_start = ");
    driver::uart::decimal(addr as u64);
    driver::uart::puts("\n");

    let addr = unsafe { &mut __stack_el2_start as *mut u64 as u64 };
    driver::uart::puts("__stack_el2_start = ");
    driver::uart::decimal(addr as u64);
    driver::uart::puts("\n");

    let addr = unsafe { &mut _end as *mut u64 as u64 };
    driver::uart::puts("_end              = ");
    driver::uart::decimal(addr as u64);
    driver::uart::puts("\n");
}

pub fn init() -> Option<VMTables> {
    print_addr();

    // check for 4k granule and at least 36 bits physical address bus
    let mut mmfr: u64;
    unsafe { asm!("mrs $0, id_aa64mmfr0_el1" : "=r" (mmfr)) };
    let b = mmfr & 0xF;
    if b < 1 /* 36 bits */ {
        driver::uart::puts("ERROR: 36 bit address space not supported\n");
        return None;
    }

    if mmfr & (0xF << 24) != 0 /* 64KiB */ {
        driver::uart::puts("ERROR: 64KiB granule not supported\n");
        return None;
    }

    Some(VMTables{el2: init_el2(), el3: init_el3()} )
}

fn init_table_flat(tt: &'static mut [u64], addr: u64) -> &'static mut [u64] {
    let end = unsafe { &mut _end as *mut u64 as usize } >> 16;

    // L2 table, 4GiB + 512MiB space
    for i in 0..8 {
        tt[i] = addr + (i as u64 + 1) * 8192 * 8 | 0b11;
    }

    // L3 table, secure kernel, and hyper visor
    for i in 0..(end - 1) {
        tt[i + 8192] = (i * 64 * 1024) as u64 | 0b11 |
            FLAG_L3_AF | FLAG_L3_ISH | FLAG_L3_SH_RW_N | FLAG_L3_ATTR_MEM;
    }

    // L3 table
    for i in end..(8192 * 8 - 1) {
        tt[i + 8192] = (i * 64 * 1024) as u64 | 0b11 |
            FLAG_L3_AF | FLAG_L3_ISH | FLAG_L3_SH_RW_RW | FLAG_L3_ATTR_NC;
    }

    let start = DRIVER_MEM_START >> 16; // div by 64 * 1024
    let end   = start + ((DRIVER_MEM_END - DRIVER_MEM_START) >> 16); // div by 64 * 1024

    // L3 table, device
    for i in start..end {
        tt[i + 8192] = (i * 64 * 1024) as u64 | 0b11 |
            FLAG_L3_XN | FLAG_L3_PXN | FLAG_L3_AF | FLAG_L3_OSH | FLAG_L3_SH_RW_RW | FLAG_L3_ATTR_DEV;
    }

    tt
}

fn get_mair() -> u64 {
    (0xFF <<  0) | // AttrIdx=0: normal, IWBWA, OWBWA, NTR
    (0x04 <<  8) | // AttrIdx=1: device, nGnRE (must be OSH too)
    (0x44 << 16)   // AttrIdx=2: non cacheable
}

fn get_tcr() -> u64 {
    let mut mmfr: u64;
    unsafe { asm!("mrs $0, id_aa64mmfr0_el1" : "=r" (mmfr)) };
    let b = mmfr & 0xF;

    1 << 31 | // Res1
    1 << 23 | // Res1
    b << 16 |
    1 << 14 | // 64KiB granule
    3 << 12 | // inner shadable
    1 << 10 | // Normal memory, Outer Write-Back Read-Allocate Write-Allocate Cacheable.
    1 <<  8 | // Normal memory, Inner Write-Back Read-Allocate Write-Allocate Cacheable.
    22        // T0SZ = 22, 2 levels (level 2 and 3 translation tables), 2^42B (4TiB) space
}

fn update_sctlr(sctlr: u64) -> u64 {
    let sctlr =
        sctlr   |
        1 << 12 | // set I, instruction cache
//        1 <<  2 | // set C, data cache
        1;        // set M, enable MMU
    sctlr & !(
        1 << 25 | // clear EE
        1 << 19 | // clear WXN
        1 <<  3 | // clear SA
        1 <<  1   // clear A
    )
}

/// set up EL3's page table, 64KB page, level 2 and 3 translation tables,
/// assume 2MiB stack space per CPU
fn init_el3() -> &'static mut [u64] {
    let addr = unsafe { &mut __tt_el3_start as *mut u64 as u64 };
    let ptr  = addr as *mut u64;
    let tt   = unsafe { slice::from_raw_parts_mut(ptr, 8192 * 10) };
    let tt   = init_table_flat(tt, addr);

    // detect stack over flow
    let end = unsafe { &mut __stack_el3_end as *mut u64 as usize };
    let start = unsafe { &mut __stack_el3_start as *mut u64 as usize };

    // #CPU
    let nc = (start - end) >> 21; // div by 2MiB (32 pages)
    for i in 0..(nc - 1) {
        tt[(end >> 16) + i * 32 + 8192] = 0;
    }

    // mask EL2's stack
    let end = unsafe { &mut __stack_el2_end as *mut u64 as usize } >> 16; // div by 64KiB
    let start = unsafe { &mut __stack_el2_start as *mut u64 as usize } >> 16; // div by 64KiB
    for i in end..(start - 1) {
        tt[i + 8192] = 0;
    }

    // mask EL2's transition table
    let end = unsafe { &mut __tt_el2_end as *mut u64 as usize } >> 16; // div by 64KiB
    let start = unsafe { &mut __tt_el2_start as *mut u64 as usize } >> 16; // div by 64KiB
    for i in start..(end - 1) {
        tt[i + 8192] = 0;
    }

    // first, set Memory Attributes array, indexed by PT_MEM, PT_DEV, PT_NC in our example
    unsafe { asm!("msr mair_el3, $0" : : "r" (get_mair())) };

    // next, specify mapping characteristics in translate control register
    unsafe { asm!("msr tcr_el3, $0" : : "r" (get_tcr())) };

    // tell the MMU where our translation tables are.
    unsafe { asm!("msr ttbr0_el3, $0" : : "r" (addr + 1)) };

    // finally, toggle some bits in system control register to enable page translation
    let mut sctlr: u64;
    unsafe { asm!("dsb ish; isb; mrs $0, sctlr_el3" : "=r" (sctlr)) };
    sctlr = update_sctlr(sctlr);
    unsafe { asm!("msr sctlr_el3, $0; dsb sy; isb" : : "r" (sctlr)) };

    tt
}

/// set up EL2's page table, 64KB page, level 2 and 3 translation tables,
/// assume 2MiB stack space per CPU
fn init_el2() -> &'static mut [u64] {
    let addr = unsafe { &mut __tt_el2_start as *mut u64 as u64 };
    let ptr  = addr as *mut u64;
    let tt   = unsafe { slice::from_raw_parts_mut(ptr, 8192 * 10) };
    let tt   = init_table_flat(tt, addr);

    // detect stack over flow
    let end = unsafe { &mut __stack_el2_end as *mut u64 as usize };
    let start = unsafe { &mut __stack_el2_start as *mut u64 as usize };

    // #CPU
    let nc = (start - end) >> 21; // div by 2MiB (32 pages)
    for i in 0..(nc - 1) {
        tt[(end >> 16) + i * 32 + 8192] = 0;
    }

    // mask EL3's stack
    let end = unsafe { &mut __stack_el3_end as *mut u64 as usize } >> 16; // div by 64KiB
    let start = unsafe { &mut __stack_el3_start as *mut u64 as usize } >> 16; // div by 64KiB
    for i in end..(start - 1) {
        tt[i + 8192] = 0;
    }

    // mask EL3's transition table
    let end = unsafe { &mut __tt_el3_end as *mut u64 as usize } >> 16; // div by 64KiB
    let start = unsafe { &mut __tt_el3_start as *mut u64 as usize } >> 16; // div by 64KiB
    for i in start..(end - 1) {
        tt[i + 8192] = 0;
    }

    // first, set Memory Attributes array, indexed by PT_MEM, PT_DEV, PT_NC in our example
    unsafe { asm!("msr mair_el2, $0" : : "r" (get_mair())) };

    // next, specify mapping characteristics in translate control register
    unsafe { asm!("msr tcr_el2, $0" : : "r" (get_tcr())) };

    // tell the MMU where our translation tables are.
    unsafe { asm!("msr ttbr0_el2, $0" : : "r" (addr + 1)) };

    // finally, toggle some bits in system control register to enable page translation
    let mut sctlr: u64;
    unsafe { asm!("dsb ish; isb; mrs $0, sctlr_el2" : "=r" (sctlr)) };
    sctlr = update_sctlr(sctlr);
    unsafe { asm!("msr sctlr_el2, $0; dsb sy; isb" : : "r" (sctlr)) };

    tt
}