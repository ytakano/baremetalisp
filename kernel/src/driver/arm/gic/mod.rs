use core::ptr::{read_volatile, write_volatile};

pub mod v2;

// Constants to categorise interrupts
pub(crate) const MIN_SGI_ID: u32 = 0;
pub(crate) const MIN_SEC_SGI_ID: u32 = 8;
pub(crate) const MIN_PPI_ID: u32 = 16;
pub(crate) const MIN_SPI_ID: u32 = 32;
pub(crate) const MAX_SPI_ID: u32 = 1019;

// Mask for the priority field common to all GIC interfaces
pub(crate) const GIC_PRI_MASK: u32 = 0xff;

// Mask for the configuration field common to all GIC interfaces */
pub(crate) const GIC_CFG_MASK: u32 = 0x3;

// Highest possible interrupt priorities
pub(crate) const GIC_HIGHEST_SEC_PRIORITY: u32 = 0x00;
pub(crate) const GIC_HIGHEST_NS_PRIORITY: u32 = 0x80;

// Common GIC Distributor interface register constants
pub(crate) const PIDR2_ARCH_REV_SHIFT: u32 = 4;
pub(crate) const PIDR2_ARCH_REV_MASK: u32 = 0xf;

// GIC revision as reported by PIDR2.ArchRev register field
pub(crate) const ARCH_REV_GICV1: u32 = 0x1;
pub(crate) const ARCH_REV_GICV2: u32 = 0x2;
pub(crate) const ARCH_REV_GICV3: u32 = 0x3;
pub(crate) const ARCH_REV_GICV4: u32 = 0x4;

pub(crate) const IGROUPR_SHIFT: u32 = 5;
pub(crate) const ISENABLER_SHIFT: u32 = 5;
pub(crate) const ICENABLER_SHIFT: u32 = ISENABLER_SHIFT;
pub(crate) const ISPENDR_SHIFT: u32 = 5;
pub(crate) const ICPENDR_SHIFT: u32 = ISPENDR_SHIFT;
pub(crate) const ISACTIVER_SHIFT: u32 = 5;
pub(crate) const ICACTIVER_SHIFT: u32 = ISACTIVER_SHIFT;
pub(crate) const IPRIORITYR_SHIFT: u32 = 2;
pub(crate) const ITARGETSR_SHIFT: u32 = 2;
pub(crate) const ICFGR_SHIFT: u32 = 4;
pub(crate) const NSACR_SHIFT: u32 = 4;

// Common GIC Distributor interface register offsets
const GICD_CTLR: usize = 0x0;
const GICD_TYPER: usize = 0x4;
const GICD_IIDR: usize = 0x8;
const GICD_IGROUPR: usize = 0x80;
const GICD_ISENABLER: usize = 0x100;
const GICD_ICENABLER: usize = 0x180;
const GICD_ISPENDR: usize = 0x200;
const GICD_ICPENDR: usize = 0x280;
const GICD_ISACTIVER: usize = 0x300;
const GICD_ICACTIVER: usize = 0x380;
const GICD_IPRIORITYR: usize = 0x400;
const GICD_ICFGR: usize = 0xc00;
const GICD_NSACR: usize = 0xe00;

// GICD_CTLR bit definitions
pub(crate) const CTLR_ENABLE_G0_SHIFT: u32 = 0;
pub(crate) const CTLR_ENABLE_G0_MASK: u32 = 0x1;
pub(crate) const CTLR_ENABLE_G0_BIT: u32 = 1;

// GICD_TYPER shifts and masks
pub(crate) const TYPER_IT_LINES_NO_SHIFT: u32 = 0;
pub(crate) const TYPER_IT_LINES_NO_MASK: u32 = 0x1f;

// Value used to initialize Normal world interrupt priorities four at a time
pub(crate) const GICD_IPRIORITYR_DEF_VAL: u32 = GIC_HIGHEST_NS_PRIORITY
    | (GIC_HIGHEST_NS_PRIORITY << 8)
    | (GIC_HIGHEST_NS_PRIORITY << 16)
    | (GIC_HIGHEST_NS_PRIORITY << 24);

/// GIC Distributor interface register accessors that are common to GICv3 & GICv2
pub(crate) fn gicd_read_ctlr(base: usize) -> u32 {
    let ptr = (base + GICD_CTLR) as *const u32;
    unsafe { read_volatile(ptr) }
}

pub(crate) fn gicd_write_ctlr(base: usize, val: u32) {
    let ptr = (base + GICD_CTLR) as *mut u32;
    unsafe {
        write_volatile(ptr, val);
    }
}

pub(crate) fn gicd_read_typer(base: usize) -> u32 {
    let ptr = (base + GICD_TYPER) as *const u32;
    unsafe { read_volatile(ptr) }
}

/// Accessor to read the GIC Distributor IGROUPR corresponding to the interrupt
/// `id`, 32 interrupt ids at a time.
pub(crate) fn gicd_read_igroupr(base: usize, id: u32) -> u32 {
    let n = id >> IGROUPR_SHIFT;
    let ptr = (base + GICD_IGROUPR + (n << 2) as usize) as *const u32;
    unsafe { read_volatile(ptr) }
}

/// Accessor to write the GIC Distributor IGROUPR corresponding to the
/// interrupt `id`, 32 interrupt IDs at a time.
pub(crate) fn gicd_write_igroupr(base: usize, id: u32, val: u32) {
    let n = id >> IGROUPR_SHIFT;
    let ptr = (base + GICD_IGROUPR + (n << 2) as usize) as *mut u32;
    unsafe {
        write_volatile(ptr, val);
    }
}

/// Accessor to write the GIC Distributor IPRIORITYR corresponding to the
/// interrupt `id`, 4 interrupt IDs at a time.
pub(crate) fn gicd_write_ipriorityr(base: usize, id: u32, val: u32) {
    let n = id >> IPRIORITYR_SHIFT;
    let ptr = (base + GICD_IPRIORITYR + (n << 2) as usize) as *mut u32;
    unsafe {
        write_volatile(ptr, val);
    }
}

/// Accessor to read the GIC Distributor ICGFR corresponding to the
/// interrupt `id`, 16 interrupt IDs at a time.
pub(crate) fn gicd_read_icfgr(base: usize, id: u32) -> u32 {
    let n = id >> ICFGR_SHIFT;
    let ptr = (base + GICD_ICFGR + (n << 2) as usize) as *const u32;
    unsafe { read_volatile(ptr) }
}

/// Accessor to write the GIC Distributor ICFGR corresponding to the
/// interrupt `id`, 16 interrupt IDs at a time.
pub(crate) fn gicd_write_icfgr(base: usize, id: u32, val: u32) {
    let n = id >> ICFGR_SHIFT;
    let ptr = (base + GICD_ICFGR + (n << 2) as usize) as *mut u32;
    unsafe {
        write_volatile(ptr, val);
    }
}

/// Accessor to write the GIC Distributor ISENABLER corresponding to the
/// interrupt `id`, 32 interrupt IDs at a time.
pub(crate) fn gicd_write_isenabler(base: usize, id: u32, val: u32) {
    let n = id >> ISENABLER_SHIFT;
    let ptr = (base + GICD_ISENABLER + (n << 2) as usize) as *mut u32;
    unsafe { write_volatile(ptr, val) }
}

/// Accessor to write the GIC Distributor ICENABLER corresponding to the
/// interrupt `id`, 32 interrupt IDs at a time.
pub(crate) fn gicd_write_icenabler(base: usize, id: u32, val: u32) {
    let n = id >> ICENABLER_SHIFT;
    let ptr = (base + GICD_ICENABLER + (n << 2) as usize) as *mut u32;
    unsafe { write_volatile(ptr, val) }
}

// GIC Distributor functions for accessing the GIC registers
// corresponding to a single interrupt ID. These functions use bitwise
// operations or appropriate register accesses to modify or return
// the bit-field corresponding the single interrupt ID.

pub(crate) fn gicd_clr_igroupr(base: usize, id: u32) {
    let bit_num = id & ((1 << IGROUPR_SHIFT) - 1);
    let reg_val = gicd_read_igroupr(base, id);
    gicd_write_igroupr(base, id, reg_val & !(1 << bit_num));
}

pub(crate) fn gicd_set_ipriorityr(base: usize, id: u32, pri: u32) {
    let val = (pri & GIC_PRI_MASK) as u8;
    let ptr = (base + GICD_IPRIORITYR + id as usize) as *mut u8;
    unsafe {
        write_volatile(ptr, val);
    }
}

pub(crate) fn gicd_set_icfgr(base: usize, id: u32, cfg: u32) {
    // Interrupt configuration is a 2-bit field
    let bit_num = id & ((1 << ICFGR_SHIFT) - 1);
    let bit_shift = bit_num << 1;

    let reg_val = gicd_read_icfgr(base, id);

    // Clear the field, and insert required configuration
    let reg_val = reg_val & !(GIC_CFG_MASK << bit_shift);
    let reg_val = reg_val | ((cfg & GIC_CFG_MASK) << bit_shift);

    gicd_write_icfgr(base, id, reg_val);
}

pub(crate) fn gicd_set_isenabler(base: usize, id: u32) {
    let bit_num = id & ((1 << ISENABLER_SHIFT) - 1);
    gicd_write_isenabler(base, id, 1 << bit_num);
}
