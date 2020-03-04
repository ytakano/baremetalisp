use super::el;

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
        Some(sctlr & 1 == 1)
    } else {
        None
    }
}

// 64KB page
// level 2 and 3 translation tables

const PAGESIZE: u64 = 64 * 1024;

#[repr(align(4096))]
struct TT<T>(T);

const L2ENTRY: TT<[u64; 8192]> = TT([0; 8192]);

// level 3 table for secure monitor
const L3ENTRY_SM: TT<[u64; 8192]> = TT([0; 8192]);

// level 3 table for EL2
const L3ENTRY_USER: TT<[u64; 8192 * 8]> = TT([0; 8192 * 8]);

/// 64KB page, level 2 and 3 translation tables
///
/// ## memory map
///
/// PAGESIZE = 64 * 1024
/// mmax     = if memsize == 4GiB then memsize - (688 * PAGESIZE) else memsize
///
/// physical                  | virtual                             | for what         | #pages (size)
/// --------------------------|-------------------------------------|------------------|-----------------
///          0 ... 0x03ffffff |        2^40 ... 0x03ffffff + 2^40   | for EL3 (static) |  1024 (64MiB)
/// 0x04000000 ... mmax - 1   |           0 ... mmax - 1            | for EL2          | 64847
/// 0xfd500000 ... 0xffffffff |  0xfd500000 ... 0xffffffff          | devices (static) |   688
/// 0x04000000 ... mmax - 1   |        2^41 ... 0x3fffffff + 2^41   | secure memory    | 16384 ( 1GiB)
/// 0x04000000 ... mmax - 1   | 2^41 + 2^32 ... 2^41 + 2^32 + 65535 | shared memory    |     1 (64KiB)
///
pub fn init(memsize: usize) -> () {
    let mmax: u64 = if memsize == 4 * 1024 * 1024 * 1024 {
        memsize as u64 - (688 * PAGESIZE)
    } else {
        memsize as u64
    };
}