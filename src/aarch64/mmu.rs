use core::intrinsics::volatile_store;
use core::intrinsics::volatile_load;

use core::slice;

use super::el;
use crate::driver;

extern "C" {
    static mut _end: u64;
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


#[cfg(feature = "raspi3")]
pub const DRIVER_MEM_START: usize =  0x3C000000;

#[cfg(feature = "raspi3")]
pub const DRIVER_MEM_END:   usize =  0x40000000;

#[cfg(feature = "raspi4")]
pub const DRIVER_MEM_START: usize =  0xfd500000;

#[cfg(feature = "raspi4")]
pub const DRIVER_MEM_END:   usize = 0x100000000;

/// 64KB page, level 2 and 3 translation tables
///
/// ## memory map
///
/// PAGESIZE = 64 * 1024
/// mmax     = if memsize == 4GiB then memsize - (688 * PAGESIZE) else memsize
///
/// physical                  | virtual                                | for what         | #pages (size)
/// --------------------------|----------------------------------------|------------------|-----------------
///          0 ... 0x03ffffff |        2^40 ... 0x03ffffff + 2^40      | for EL3 (static) |  1024 ( 64MiB)
/// 0x04000000 ... mmax - 1   |           0 ... mmax - 1               | for EL2          | 64847
/// 0xfd500000 ... 0xffffffff |  0xfd500000 ... 0xffffffff             | devices (static) |   688
/// 0x04000000 ... mmax - 1   |        2^41 ... 0x3fffffff + 2^41      | secure memory    | 16384 (  1GiB)
/// 0x04000000 ... mmax - 1   | 2^41 + 2^32 ... 2^41 + 2^32 + 2^17 - 1 | shared memory    |     2 (128KiB)
///
pub fn init(memsize: usize) -> () {
    let mmax: u64 = if memsize == 4 * 1024 * 1024 * 1024 {
        memsize as u64 - (688 * PAGESIZE)
    } else {
        memsize as u64
    };

    let mut addr = unsafe { &mut _end as *mut u64 as u64 };
    if addr % PAGESIZE != 0 {
        addr += PAGESIZE - (addr % PAGESIZE);
    }
    let ptr = addr as *mut u64;
    let tt  = unsafe { slice::from_raw_parts_mut(ptr, 8192 * 13) };

    for i in 0..(8192 * 13 - 1) {
        tt[i] = 0;
    }

    for i in 0..7 { // L2 table
        tt[i] = addr + (i as u64 + 1) * 8192 * 8 | 0b11 | FLAG_L2_NS;
    }

    for i in 0..8191 { // L3 table
        tt[i + 8192] = (i * 64 * 1024) as u64 | 0b11 |
            FLAG_L3_NS | FLAG_L3_AF | FLAG_L3_ISH | FLAG_L3_SH_RW_RW | FLAG_L3_ATTR_MEM;
    }

    let start = DRIVER_MEM_START / 64 / 1024;
    let end   = start + (DRIVER_MEM_END - DRIVER_MEM_START) / 64 / 1024;

    for i in start..end { // device
        tt[i + 8192] = (i * 64 * 1024) as u64 | 0b11 |
            FLAG_L3_NS | FLAG_L3_XN | FLAG_L3_PXN | FLAG_L3_AF | FLAG_L3_OSH | FLAG_L3_SH_RW_RW | FLAG_L3_ATTR_DEV;
    }

    // check for 4k granule and at least 36 bits physical address bus
    let mut mmfr: u64;
    unsafe { asm!("mrs $0, id_aa64mmfr0_el1" : "=r" (mmfr)) };
    let b = mmfr & 0xF;
    if b < 1 /* 36 bits */ {
        driver::uart::puts("ERROR: 36 bit address space not supported\n");
        return;
    }

    if mmfr & (0xF << 24) != 0 /* 64KiB */ {
        driver::uart::puts("ERROR: 64KiB granule not supported\n");
        return;
    }

    // first, set Memory Attributes array, indexed by PT_MEM, PT_DEV, PT_NC in our example
    let mair: u64 = (0xFF <<  0) | // AttrIdx=0: normal, IWBWA, OWBWA, NTR
                    (0x04 <<  8) | // AttrIdx=1: device, nGnRE (must be OSH too)
                    (0x44 << 16);  // AttrIdx=2: non cacheable
    unsafe { asm!("msr mair_el2, $0" : : "r" (mair)) };

    // next, specify mapping characteristics in translate control register
    let tcr: u64 = 1 << 31 | // Res1
                   1 << 23 | // Res1
                   b << 16 |
                   1 << 14 | // 64KiB granule
                   3 << 12 | // inner shadable
                   1 << 10 | // Normal memory, Outer Write-Back Read-Allocate Write-Allocate Cacheable.
                   1 <<  8 | // Normal memory, Inner Write-Back Read-Allocate Write-Allocate Cacheable.
                   32;       // T0SZ = 22, 2 levels (level 2 and 3 translation tables), 2^42B (4TiB) space
    unsafe { asm!("msr tcr_el2, $0" : : "r" (tcr)) };

    // tell the MMU where our translation tables are.
    unsafe { asm!("msr ttbr0_el2, $0" : : "r" (addr + 1)) };

    // Enables data coherency with other cores in the cluster.
    let mut extend: u64;
    unsafe { asm!("mrs $0, S3_1_C15_C2_1" : "=r" (extend)) };
    extend |= 1 << 6; // the SMP bit
    unsafe { asm!("msr S3_1_C15_C2_1, $0" : : "r" (extend)) };

    // finally, toggle some bits in system control register to enable page translation
    let mut sctlr: u64;
    unsafe { asm!("dsb ish; isb; mrs $0, sctlr_el2" : "=r" (sctlr)) };

    driver::uart::puts("MMU: read sctlr\n");

    sctlr |=
        1 << 12 | // no effect on the Cacheability of instruction access to Normal memory
        1 <<  2 | // no effect on the Cacheability
        1;        // set M, enable MMU
    sctlr &= !(
        1 << 25 | // clear EE
        1 << 19 | // clear WXN
        1 <<  3 | // clear SA
        1 <<  1   // clear A
    );

    unsafe { asm!("msr sctlr_el2, $0; dsb sy; isb" : : "r" (sctlr)) };

//    sctlr -= 1;
//    unsafe { asm!("msr sctlr_el2, $0; dsb sy; isb" : : "r" (sctlr)) };

    driver::uart::puts("MMU: write sctlr\n");
}