use core::slice;

use super::el;
use crate::driver;

//-----------------------------------------------------------------------------
// Raspberry Pi 3
#[cfg(any(feature = "raspi3"))]
pub const DEVICE_MEM_START: u64 =  0x3C000000;

#[cfg(any(feature = "raspi3"))]
pub const DEVICE_MEM_END:   u64 =  0x40000000;

#[cfg(feature = "raspi3")]
pub const NUM_CPU:          u64 = 4;
//-----------------------------------------------------------------------------

//-----------------------------------------------------------------------------
// Raspberry Pi 4
#[cfg(feature = "raspi4")]
pub const DEVICE_MEM_START: u64 = 0x0fd000000; // maybe...

#[cfg(feature = "raspi4")]
pub const DEVICE_MEM_END:   u64 = 0x100000000; // maybe...

#[cfg(feature = "raspi4")]
pub const NUM_CPU:          u64 = 4;
//-----------------------------------------------------------------------------

//-----------------------------------------------------------------------------
// PINE64
#[cfg(feature = "pine64")]
pub const DEVICE_MEM_START: u64 =  0x01C00000;

#[cfg(feature = "pine64")]
pub const DEVICE_MEM_END:   u64 =  0x01F10000;

#[cfg(feature = "pine64")]
pub const NUM_CPU:          u64 = 4;
//-----------------------------------------------------------------------------


pub const EL1_ADDR_OFFSET: u64 = 0x3FFFFF << 42;

// level 2 table x 1 (for 4TiB space)
// level 3 table x 8 (for 512MiB x 8 = 4GiB space)
pub const FIRM_LV2_TABLE_NUM: usize = 1;
pub const FIRM_LV3_TABLE_NUM: usize = 8;
pub const FIRM_TABLE_NUM: usize = FIRM_LV2_TABLE_NUM + FIRM_LV3_TABLE_NUM;

// level 2 table x 1 (for 4TiB space)
// level 3 table x 8 (for 512MiB x 8 = 4GiB space)
pub const KERN_TTBR0_LV2_TABLE_NUM: usize = 1;
pub const KERN_TTBR0_LV3_TABLE_NUM: usize = 8;
pub const KERN_TTBR0_TABLE_NUM: usize = KERN_TTBR0_LV2_TABLE_NUM + KERN_TTBR0_LV3_TABLE_NUM;

// level 2 table x 1 (for 4TiB space)
// level 3 table x 1 (for 512MiB space)
pub const KERN_TTBR1_LV2_TABLE_NUM: usize = 1;
pub const KERN_TTBR1_LV3_TABLE_NUM: usize = 1;
pub const KERN_TTBR1_TABLE_NUM: usize = KERN_TTBR1_LV2_TABLE_NUM + KERN_TTBR1_LV3_TABLE_NUM;

extern "C" {
    static __ram_start: u64;
    static __free_mem_start: u64;
    static __data_start: u64;
    static __data_end: u64;
    static __bss_start: u64;
    static __bss_end: u64;
    static __stack_firm_end: u64;
    static __stack_firm_start: u64;
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
// see get_mair()
const FLAG_L3_ATTR_MEM: u64 = 0     ; // normal memory
const FLAG_L3_ATTR_DEV: u64 = 1 << 2; // device MMIO
const FLAG_L3_ATTR_NC:  u64 = 2 << 2; // non-cachable

// transition table
pub struct TTable {
    tt_lv2: &'static mut [u64],
    tt_lv3: &'static mut [u64],
    num_lv2: usize,
    num_lv3: usize
}

pub struct VMTables {
    el1: &'static mut [u64],
    firm: &'static mut [u64],
}

// logical address information
pub struct Addr {
    // must be same as physical
    pub no_cache_start: u64,
    pub no_cache_end: u64,
    pub tt_firm_start: u64,
    pub tt_firm_end: u64,
    pub tt_el1_ttbr0_start: u64,
    pub tt_el1_ttbr0_end: u64,
    pub tt_el1_ttbr1_start: u64,
    pub tt_el1_ttbr1_end: u64,

    pub stack_size: u64,

    // independent from physical
    pub stack_el1_end: u64,
    pub stack_el1_start: u64,
    pub stack_el0_end: u64,
    pub stack_el0_start: u64,
    pub el0_heap_start: u64,
    pub el0_heap_end: u64
}

impl Addr {
    pub fn new() -> Addr {
        let no_cache_start = unsafe { &__free_mem_start as *const u64 as u64 };
        let no_cache_end   = no_cache_start + PAGESIZE;

        // MMU's transition table for firmware
        let tt_firm_start = no_cache_end;
        let tt_firm_end   = tt_firm_start + PAGESIZE * FIRM_TABLE_NUM as u64;

        // MMU's transition table #0 for EL1
        let tt_el1_ttbr0_start = tt_firm_end;
        let tt_el1_ttbr0_end   = tt_el1_ttbr0_start + PAGESIZE * KERN_TTBR0_TABLE_NUM as u64;

        // MMU's transition table #1 for EL1
        // level 2 table x 1 (for 4TiB space)
        // level 3 table x 1 (for 512MiB space)
        let tt_el1_ttbr1_start = tt_el1_ttbr0_end;
        let tt_el1_ttbr1_end   = tt_el1_ttbr1_start + PAGESIZE * KERN_TTBR1_TABLE_NUM as u64;

        // 2MiB stack x NUM_CPU
        let stack_size = 32 * PAGESIZE * NUM_CPU;

        // EL1's stack
        let stack_el1_end   = tt_el1_ttbr1_end;
        let stack_el1_start = stack_el1_end + stack_size;

        // EL0's stack
        let stack_el0_end   = stack_el1_start;
        let stack_el0_start = stack_el0_end + stack_size;

        // heap memory for EL0
        let el0_heap_start = stack_el0_start;
        let el0_heap_end   = el0_heap_start + PAGESIZE * 1024; // 64MiB

        Addr{
            no_cache_start: no_cache_start,
            no_cache_end: no_cache_end,
            tt_firm_start: tt_firm_start,
            tt_firm_end: tt_firm_end,
            tt_el1_ttbr0_start: tt_el1_ttbr0_start,
            tt_el1_ttbr0_end: tt_el1_ttbr0_end,
            tt_el1_ttbr1_start: tt_el1_ttbr1_start,
            tt_el1_ttbr1_end: tt_el1_ttbr1_end,
            stack_size: stack_size,
            stack_el1_end: stack_el1_end,
            stack_el1_start: stack_el1_start,
            stack_el0_end: stack_el0_end,
            stack_el0_start: stack_el0_start,
            el0_heap_start: el0_heap_start,
            el0_heap_end: el0_heap_end
        }
    }

    fn print(&self) {
        let addr = unsafe { &__ram_start as *const u64 as u64 };
        driver::uart::puts("__ram_start        = 0x");
        driver::uart::hex(addr);
        driver::uart::puts("\n");

        let addr = unsafe { &__data_start as *const u64 as u64 };
        driver::uart::puts("__data_start       = 0x");
        driver::uart::hex(addr);
        driver::uart::puts("\n");

        let addr = unsafe { &__data_end as *const u64 as u64 };
        driver::uart::puts("__data_end         = 0x");
        driver::uart::hex(addr);
        driver::uart::puts("\n");

        let addr = unsafe { &__bss_start as *const u64 as u64 };
        driver::uart::puts("__bss_start        = 0x");
        driver::uart::hex(addr);
        driver::uart::puts("\n");

        let addr = unsafe { &__stack_firm_end as *const u64 as u64 };
        driver::uart::puts("__stack_firm_end   = 0x");
        driver::uart::hex(addr);
        driver::uart::puts("\n");

        let addr = unsafe { &__stack_firm_start as *const u64 as u64 };
        driver::uart::puts("__stack_firm_start = 0x");
        driver::uart::hex(addr);
        driver::uart::puts("\n");

        driver::uart::puts("no_cache_start     = 0x");
        driver::uart::hex(self.no_cache_start as u64);
        driver::uart::puts("\n");

        driver::uart::puts("no_cache_end       = 0x");
        driver::uart::hex(self.no_cache_end as u64);
        driver::uart::puts("\n");

        driver::uart::puts("tt_firm_start      = 0x");
        driver::uart::hex(self.tt_firm_start as u64);
        driver::uart::puts("\n");

        driver::uart::puts("tt_firm_end        = 0x");
        driver::uart::hex(self.tt_firm_end as u64);
        driver::uart::puts("\n");

        driver::uart::puts("tt_el1_ttbr0_start = 0x");
        driver::uart::hex(self.tt_el1_ttbr0_start as u64);
        driver::uart::puts("\n");

        driver::uart::puts("tt_el1_ttbr0_end   = 0x");
        driver::uart::hex(self.tt_el1_ttbr0_end as u64);
        driver::uart::puts("\n");

        driver::uart::puts("tt_el1_ttbr1_start = 0x");
        driver::uart::hex(self.tt_el1_ttbr1_start as u64);
        driver::uart::puts("\n");

        driver::uart::puts("tt_el1_ttbr1_end   = 0x");
        driver::uart::hex(self.tt_el1_ttbr1_end as u64);
        driver::uart::puts("\n");

        driver::uart::puts("stack_el1_end      = 0x");
        driver::uart::hex(self.stack_el1_end as u64);
        driver::uart::puts("\n");

        driver::uart::puts("stack_el1_start    = 0x");
        driver::uart::hex(self.stack_el1_start as u64);
        driver::uart::puts("\n");

        driver::uart::puts("stack_el0_end      = 0x");
        driver::uart::hex(self.stack_el0_end as u64);
        driver::uart::puts("\n");

        driver::uart::puts("stack_el0_start    = 0x");
        driver::uart::hex(self.stack_el0_start as u64);
        driver::uart::puts("\n");

        driver::uart::puts("el0_heap_start     = 0x");
        driver::uart::hex(self.el0_heap_start as u64);
        driver::uart::puts("\n");

        driver::uart::puts("el0_heap_end       = 0x");
        driver::uart::hex(self.el0_heap_end as u64);
        driver::uart::puts("\n");
    }
}

impl TTable {
    fn new(tt_addr: u64, num_lv2 : usize, num_lv3 : usize) -> TTable {
        let ptr = tt_addr as *mut u64;
        let tt_lv2 = unsafe { slice::from_raw_parts_mut(ptr, 8192 * num_lv2) };

        let ptr = ((PAGESIZE * num_lv2 as u64) + tt_addr) as *mut u64;
        let tt_lv3 = unsafe { slice::from_raw_parts_mut(ptr, 8192 * num_lv3) };

        // initialize
        for e in tt_lv2.iter_mut() {
            *e = 0;
        }

        for e in tt_lv3.iter_mut() {
            *e = 0;
        }

        // set up level 2 tables
        for i in 0..(8192 * num_lv2) {
            if i >= num_lv3 {
                break;
            }
            tt_lv2[i] = (&tt_lv3[i * 8192] as *const u64) as u64 | 0b11;
        }

        TTable{tt_lv2: tt_lv2, tt_lv3: tt_lv3, num_lv2: num_lv2, num_lv3: num_lv3}
    }

    fn map(&mut self, vm_addr: u64, phy_addr: u64, flag: u64) {
        let lv2idx = ((vm_addr >> 29) & 8191) as usize;
        let lv3idx = ((vm_addr >> 16) & 8191) as usize;

        if lv2idx >= self.num_lv3 {
            // memory access error
            return;
        }

        let e = phy_addr & !((1 << 16) - 1) | flag;
        let idx = lv2idx * 8192 + lv3idx;
        self.tt_lv3[idx] = e as u64;
    }

    fn unmap(&mut self, vm_addr: u64) {
        let lv2idx = ((vm_addr >> 29) & 8191) as usize;
        let lv3idx = ((vm_addr >> 16) & 8191) as usize;

        if lv2idx >= self.num_lv3 {
            // memory access error
            return;
        }

        let idx = lv2idx * 8192 + lv3idx;
        self.tt_lv3[idx] = 0;
    }
}

pub fn enabled() -> Option<bool> {
    let mut sctlr: u32;

    let el = el::get_current_el();
    if el == 1 {
        unsafe { llvm_asm!("mrs $0, SCTLR_EL1" : "=r"(sctlr)) };
        Some(sctlr & 1 == 1)
    } else if el == 2 {
        unsafe { llvm_asm!("mrs $0, SCTLR_EL2" : "=r"(sctlr)) };
        Some(sctlr & 1 == 1)
    } else if el == 3 {
        unsafe { llvm_asm!("mrs $0, SCTLR_EL3" : "=r"(sctlr)) };
        Some((sctlr & 1) == 1)
    } else {
        None
    }
}

fn get_sctlr() -> u32 {
    let mut sctlr: u32 = 0;
    let el = el::get_current_el();
    if el == 1 {
        unsafe { llvm_asm!("mrs $0, SCTLR_EL1" : "=r"(sctlr)) };
    } else if el == 2 {
        unsafe { llvm_asm!("mrs $0, SCTLR_EL2" : "=r"(sctlr)) };
    } else if el == 3 {
        unsafe { llvm_asm!("mrs $0, SCTLR_EL3" : "=r"(sctlr)) };
    }

    sctlr
}

fn set_sctlr(sctlr: u32) {
    let el = el::get_current_el();
    if el == 1 {
        unsafe { llvm_asm!("msr SCTLR_EL1, $0" : : "r"(sctlr)) };
    } else if el == 2 {
        unsafe { llvm_asm!("msr SCTLR_EL2, $0" : : "r"(sctlr)) };
    } else if el == 3 {
        unsafe { llvm_asm!("msr SCTLR_EL3, $0" : : "r"(sctlr)) };
    }
}

pub fn init() -> Option<(Addr, TTable, (TTable, TTable))> {
    let addr = Addr::new();

    addr.print();

    // check for 64KiB granule and at least 36 bits physical address bus
    let mut mmfr: u64;
    unsafe { llvm_asm!("mrs $0, id_aa64mmfr0_el1" : "=r" (mmfr)) };
    let b = mmfr & 0xF;
    if b < 1 /* 36 bits */ {
        driver::uart::puts("ERROR: 36 bit address space not supported\n");
        return None;
    }

    if mmfr & (0xF << 24) != 0 /* 64KiB */ {
        driver::uart::puts("ERROR: 64KiB granule not supported\n");
        return None;
    }

    let table_firm = if el::get_current_el() == 2 {
        init_el2(&addr)
    } else {
        init_el3(&addr)
    };

    let table_el1 = init_el1(&addr);

    Some((addr, table_firm, table_el1))
}

fn get_mair() -> u64 {
    (0xFF <<  0) | // AttrIdx=0: normal, IWBWA, OWBWA, NTR
    (0x04 <<  8) | // AttrIdx=1: device, nGnRE (must be OSH too)
    (0x44 << 16)   // AttrIdx=2: non cacheable
}

/// for TCR_EL2 and TCR_EL2
fn get_tcr() -> u64 {
    let mut mmfr: u64;
    unsafe { llvm_asm!("mrs $0, id_aa64mmfr0_el1" : "=r" (mmfr)) };
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
        1 << 44 | // set DSSBS, enable speculative load and store
        1 << 12 | // set I, instruction cache
        1 <<  2 | // set C, data cache
        1;        // set M, enable MMU
    sctlr & !(
        1 << 25 | // clear EE
        1 << 19 | // clear WXN
        1 <<  3 | // clear SA
        1 <<  1   // clear A
    )
}

fn init_firm(addr: &Addr) -> TTable {
    let mut table = TTable::new(addr.tt_firm_start, FIRM_LV2_TABLE_NUM, FIRM_LV3_TABLE_NUM);

    // map .init and .text section
    let mut ram_start = unsafe { &__ram_start as *const u64 as u64 };
    let data_start = unsafe { &__data_start as *const u64 as u64 };
    let flag = FLAG_L3_AF | FLAG_L3_ISH | FLAG_L3_SH_R_R | FLAG_L3_ATTR_MEM | 0b11;
    while ram_start < data_start {
        table.map(ram_start, ram_start, flag);
        ram_start += PAGESIZE;
    }

    // map .bss section
    let mut bss_start = unsafe { &__bss_start as *const u64 as u64 };
    let end = unsafe { &__stack_firm_end as *const u64 as u64 };
    let flag = FLAG_L3_XN | FLAG_L3_PXN | FLAG_L3_AF | FLAG_L3_ISH | FLAG_L3_SH_RW_RW | FLAG_L3_ATTR_MEM | 0b11;
    while bss_start < end {
        table.map(bss_start, bss_start, flag);
        bss_start += PAGESIZE;
    }

    // map firmware stack
    let mut stack_end = unsafe { &__stack_firm_end as *const u64 as u64 };
    let stack_start = unsafe { &__stack_firm_start as *const u64 as u64 };
    let flag = FLAG_L3_XN | FLAG_L3_PXN | FLAG_L3_AF | FLAG_L3_ISH | FLAG_L3_SH_RW_N | FLAG_L3_ATTR_MEM | 0b11;
    while stack_end < stack_start {
        table.map(stack_end, stack_end, flag);
        stack_end += PAGESIZE;
    }

    // map non cached memory
    let mut no_cache_start = addr.no_cache_start;
    let flag = FLAG_L3_XN | FLAG_L3_PXN | FLAG_L3_AF | FLAG_L3_ISH | FLAG_L3_SH_RW_N | FLAG_L3_ATTR_MEM | 0b11;
    while no_cache_start < addr.no_cache_end {
        table.map(no_cache_start, no_cache_start, flag);
        no_cache_start += PAGESIZE;
    }

    // map transition table for EL2
    let mut tt_firm_start = addr.tt_firm_start;
    let flag = FLAG_L3_XN | FLAG_L3_PXN | FLAG_L3_AF | FLAG_L3_ISH | FLAG_L3_SH_RW_N | FLAG_L3_ATTR_MEM | FLAG_L3_ATTR_NC | 0b11;
    while tt_firm_start < addr.tt_firm_end {
        table.map(tt_firm_start, tt_firm_start, flag);
        tt_firm_start += PAGESIZE;
    }

    // map transition table for EL1 TTBR0
    let mut tt_start = addr.tt_el1_ttbr0_start;
    let flag = FLAG_L3_XN | FLAG_L3_PXN | FLAG_L3_AF | FLAG_L3_ISH | FLAG_L3_SH_RW_N | FLAG_L3_ATTR_MEM | FLAG_L3_ATTR_NC | 0b11;
    while tt_start < addr.tt_el1_ttbr0_end {
        table.map(tt_start, tt_start, flag);
        tt_start += PAGESIZE;
    }

    // map transition table for EL1 TTBR1
    let mut tt_start = addr.tt_el1_ttbr1_start;
    let flag = FLAG_L3_XN | FLAG_L3_PXN | FLAG_L3_AF | FLAG_L3_ISH | FLAG_L3_SH_RW_N | FLAG_L3_ATTR_MEM | FLAG_L3_ATTR_NC | 0b11;
    while tt_start < addr.tt_el1_ttbr1_end {
        table.map(tt_start, tt_start, flag);
        tt_start += PAGESIZE;
    }

    // map device memory
    let mut device_addr = DEVICE_MEM_START;
    let flag = FLAG_L3_NS | FLAG_L3_XN | FLAG_L3_PXN | FLAG_L3_AF | FLAG_L3_OSH | FLAG_L3_SH_RW_RW | FLAG_L3_ATTR_DEV | 0b11;
    while device_addr < DEVICE_MEM_END {
        table.map(device_addr, device_addr, flag);
        device_addr += PAGESIZE;
    }

    table
}

/// set up EL3's page table, 64KB page, level 2 and 3 translation tables,
/// assume 2MiB stack space per CPU
fn init_el3(addr: &Addr) -> TTable {
    let table = init_firm(addr);

    // first, set Memory Attributes array, indexed by PT_MEM, PT_DEV, PT_NC in our example
    unsafe { llvm_asm!("msr mair_el3, $0" : : "r" (get_mair())) };

    // next, specify mapping characteristics in translate control register
    unsafe { llvm_asm!("msr tcr_el3, $0" : : "r" (get_tcr())) };

    // tell the MMU where our translation tables are.
    unsafe { llvm_asm!("msr ttbr0_el3, $0" : : "r" (addr.tt_firm_start | 1)) };

    // finally, toggle some bits in system control register to enable page translation
    let mut sctlr: u64;
    unsafe { llvm_asm!("dsb ish; isb; mrs $0, sctlr_el3" : "=r" (sctlr)) };
    sctlr = update_sctlr(sctlr);
    unsafe { llvm_asm!("msr sctlr_el3, $0; dsb sy; isb" : : "r" (sctlr)) };

    table
}

fn init_el2(addr: &Addr) -> TTable {
    let table = init_firm(addr);

    // first, set Memory Attributes array, indexed by PT_MEM, PT_DEV, PT_NC in our example
    unsafe { llvm_asm!("msr mair_el2, $0" : : "r" (get_mair())) };

    // next, specify mapping characteristics in translate control register
    unsafe { llvm_asm!("msr tcr_el2, $0" : : "r" (get_tcr())) };

    // tell the MMU where our translation tables are.
    unsafe { llvm_asm!("msr ttbr0_el2, $0" : : "r" (addr.tt_firm_start | 1)) };

    // finally, toggle some bits in system control register to enable page translation
    let mut sctlr: u64;
    unsafe { llvm_asm!("dsb ish; isb; mrs $0, sctlr_el2" : "=r" (sctlr)) };
    sctlr = update_sctlr(sctlr);
    unsafe { llvm_asm!("msr sctlr_el2, $0; dsb sy; isb" : : "r" (sctlr)) };

    table
}

/// set up EL1's page table, 64KB page, level 2 and 3 translation tables,
/// assume 2MiB stack space per CPU
fn init_el1(addr: &Addr) -> (TTable, TTable) {
    // TTBR0: user space
    let mut table0 = TTable::new(addr.tt_el1_ttbr0_start, KERN_TTBR0_LV2_TABLE_NUM, KERN_TTBR0_LV3_TABLE_NUM);

    // map .init and .text section
    let mut ram_start = unsafe { &__ram_start as *const u64 as u64 };
    let data_start = unsafe { &__data_start as *const u64 as u64 };
    let flag = FLAG_L3_AF | FLAG_L3_ISH | FLAG_L3_SH_R_R | FLAG_L3_ATTR_MEM | 0b11;
    while ram_start < data_start {
        table0.map(ram_start, ram_start, flag);
        ram_start += PAGESIZE;
    }

    // map .bss section
    let mut bss_start = unsafe { &__bss_start as *const u64 as u64 };
    let end = unsafe { &__stack_firm_end as *const u64 as u64 };
    let flag = FLAG_L3_XN | FLAG_L3_PXN | FLAG_L3_AF | FLAG_L3_ISH | FLAG_L3_SH_RW_RW | FLAG_L3_ATTR_MEM | 0b11;
    while bss_start < end {
        table0.map(bss_start, bss_start, flag);
        bss_start += PAGESIZE;
    }

    // map userland stack
    let mut stack_end = addr.stack_el0_end;
    let flag = FLAG_L3_XN | FLAG_L3_PXN | FLAG_L3_AF | FLAG_L3_ISH | FLAG_L3_SH_RW_RW | FLAG_L3_ATTR_MEM | 0b11;
    while stack_end < addr.stack_el0_start {
        table0.map(stack_end, stack_end, flag);
        stack_end += PAGESIZE;
    }

    // map userland heap
    let mut heap_start = addr.el0_heap_start;
    let flag = FLAG_L3_XN | FLAG_L3_PXN | FLAG_L3_AF | FLAG_L3_ISH | FLAG_L3_SH_RW_RW | FLAG_L3_ATTR_MEM | 0b11;
    while heap_start < addr.el0_heap_end {
        table0.map(heap_start, heap_start, flag);
        heap_start += PAGESIZE;
    }

    // map device memory
    let mut device_addr = DEVICE_MEM_START;
    let flag = FLAG_L3_NS | FLAG_L3_XN | FLAG_L3_PXN | FLAG_L3_AF | FLAG_L3_OSH | FLAG_L3_SH_RW_RW | FLAG_L3_ATTR_DEV | 0b11;
    while device_addr < DEVICE_MEM_END {
        table0.map(device_addr, device_addr, flag);
        device_addr += PAGESIZE;
    }

    //-------------------------------------------------------------------------
    // TTBR1: kernel space
    let mut table1 = TTable::new(addr.tt_el1_ttbr1_start, KERN_TTBR1_LV2_TABLE_NUM, KERN_TTBR1_LV3_TABLE_NUM);

    // kernel stack
    let mut stack_end = addr.stack_el1_end;
    let flag = FLAG_L3_XN | FLAG_L3_PXN | FLAG_L3_AF | FLAG_L3_ISH | FLAG_L3_SH_RW_N | FLAG_L3_ATTR_MEM | 0b11;
    while stack_end < addr.stack_el1_start {
        table1.map(stack_end, stack_end, flag);
        stack_end += PAGESIZE;
    }

    // map transition table for TTBR0
    let mut tt_start = addr.tt_el1_ttbr0_start;
    let flag = FLAG_L3_XN | FLAG_L3_PXN | FLAG_L3_AF | FLAG_L3_ISH | FLAG_L3_SH_RW_N | FLAG_L3_ATTR_MEM | FLAG_L3_ATTR_NC | 0b11;
    while tt_start < addr.tt_el1_ttbr0_end {
        table1.map(tt_start, tt_start, flag);
        tt_start += PAGESIZE;
    }

    // map transition table for TTBR1
    let mut tt_start = addr.tt_el1_ttbr1_start;
    let flag = FLAG_L3_XN | FLAG_L3_PXN | FLAG_L3_AF | FLAG_L3_ISH | FLAG_L3_SH_RW_N | FLAG_L3_ATTR_MEM | FLAG_L3_ATTR_NC | 0b11;
    while tt_start < addr.tt_el1_ttbr1_end {
        table1.map(tt_start, tt_start, flag);
        tt_start += PAGESIZE;
    }

    //-------------------------------------------------------------------------

    // first, set Memory Attributes array, indexed by PT_MEM, PT_DEV, PT_NC in our example
    unsafe { llvm_asm!("msr mair_el1, $0" : : "r" (get_mair())) };

    let mut mmfr: u64;
    unsafe { llvm_asm!("mrs $0, id_aa64mmfr0_el1" : "=r" (mmfr)) };
    let b = mmfr & 0xF;

    let tcr: u64 =
         b << 32 |
         3 << 30 | // 64KiB granule, TTBR1_EL1
         3 << 28 | // inner shadable, TTBR1_EL1
         1 << 26 | // Normal memory, Outer Write-Back Read-Allocate Write-Allocate Cacheable, TTBR1_EL1
         1 << 24 | // Normal memory, Inner Write-Back Read-Allocate Write-Allocate Cacheable, TTBR1_EL1
        22 << 16 | // T1SZ = 22, 2 levels (level 2 and 3 translation tables), 2^42B (4TiB) space
         1 << 14 | // 64KiB granule
         3 << 12 | // inner shadable, TTBR0_EL1
         1 << 10 | // Normal memory, Outer Write-Back Read-Allocate Write-Allocate Cacheable, TTBR0_EL1
         1 <<  8 | // Normal memory, Inner Write-Back Read-Allocate Write-Allocate Cacheable, TTBR0_EL1
        22;        // T0SZ = 22, 2 levels (level 2 and 3 translation tables), 2^42B (4TiB) space

    // next, specify mapping characteristics in translate control register
    unsafe { llvm_asm!("msr tcr_el1, $0" : : "r" (tcr)) };

    // tell the MMU where our translation tables are.
    unsafe { llvm_asm!("msr ttbr0_el1, $0" : : "r" (addr.tt_el1_ttbr0_start | 1)) };
    unsafe { llvm_asm!("msr ttbr1_el1, $0" : : "r" (addr.tt_el1_ttbr1_start | 1)) };

    // finally, toggle some bits in system control register to enable page translation
    let mut sctlr: u64;
    unsafe { llvm_asm!("dsb ish; isb; mrs $0, sctlr_el1" : "=r" (sctlr)) };
    sctlr = update_sctlr(sctlr);
    sctlr &= !(
        1 << 4 // clear SA0
    );
    unsafe { llvm_asm!("msr sctlr_el1, $0; dsb sy; isb" : : "r" (sctlr)) };

    (table0, table1)
}