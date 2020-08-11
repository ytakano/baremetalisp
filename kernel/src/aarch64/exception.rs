use crate::driver;
use crate::el3;

#[repr(C)]
pub struct Context {
    pub x0: u64,
    pub x1: u64,
    pub x2: u64,
    pub x3: u64,
    pub x4: u64,
    pub x5: u64,
    pub x6: u64,
    pub x7: u64,
    pub x8: u64,
    pub x9: u64,
    pub x10: u64,
    pub x11: u64,
    pub x12: u64,
    pub x13: u64,
    pub x14: u64,
    pub x15: u64,
    pub x16: u64,
    pub x17: u64,
    pub x18: u64,
    pub x19: u64,
    pub x20: u64,
    pub x21: u64,
    pub x22: u64,
    pub x23: u64,
    pub x24: u64,
    pub x25: u64,
    pub x26: u64,
    pub x27: u64,
    pub x28: u64,
    pub x29: u64,
    pub x30: u64,  // link register
    pub elr: u64,  // exception link register
    pub spsr: u32, // saved program status register
    _unused: [u8; 12],
}

//------------------------------------------------------------------------------

pub fn get_esr_el3() -> u32 {
    let esr: u64;
    unsafe { asm!("mrs {}, esr_el3", lateout(reg) esr) };
    esr as u32
}

// from the current EL using the current SP0
#[no_mangle]
pub fn curr_el_sp0_sync_el3(__ctx: *mut Context) {
    driver::uart::puts("EL3 exception: SP0 Sync\n");
}

#[no_mangle]
pub fn curr_el_sp0_irq_el3(__ctx: *mut Context) {
    driver::uart::puts("EL3 exception: SP0 IRQ\n");
}

#[no_mangle]
pub fn curr_el_sp0_fiq_el3(__ctx: *mut Context) {
    driver::uart::puts("EL3 exception: SP0 FIQ\n");
}

#[no_mangle]
pub fn curr_el_sp0_serror_el3(__ctx: *mut Context) {
    driver::uart::puts("EL3 exception: SP0 Error\n");
}

// from the current EL using the current SP
#[no_mangle]
pub fn curr_el_spx_sync_el3(ctx: *mut Context) {
    let r = unsafe { &*ctx };
    driver::uart::puts("EL3 exception: SPX Sync\nELR = 0x");
    driver::uart::hex(r.elr);
    driver::uart::puts("\nSPSR = 0x");
    driver::uart::hex(r.spsr as u64);
    driver::uart::puts("\nESR = 0x");
    driver::uart::hex(get_esr_el3() as u64);
    driver::uart::puts("\n");
}

#[no_mangle]
pub fn curr_el_spx_irq_el3(_ctx: *mut Context) {
    driver::uart::puts("EL3 exception: SPX IRQ\n");
}

#[no_mangle]
pub fn curr_el_spx_fiq_el3(_ctx: *mut Context) {
    driver::uart::puts("EL3 exception: SPX FIQ\n");
}

#[no_mangle]
pub fn curr_el_spx_serror_el3(ctx: *mut Context) {
    let r = unsafe { &*ctx };
    driver::uart::puts("EL3 exception: SPX Error\nELR = 0x");
    driver::uart::hex(r.elr);
    driver::uart::puts("\nSPSR = 0x");
    driver::uart::hex(r.spsr as u64);
    driver::uart::puts("\nESR = 0x");
    driver::uart::hex(get_esr_el3() as u64);
    driver::uart::puts("\n");
}

// from lower EL (AArch64)
#[no_mangle]
pub fn lower_el_aarch64_sync_el3(ctx: *mut Context) {
    // secure monitor call
    el3::handle_smc64(unsafe { &*ctx });
}

#[no_mangle]
pub fn lower_el_aarch64_irq_el3(_ctx: *mut Context) {}

#[no_mangle]
pub fn lower_el_aarch64_fiq_el3(_ctx: *mut Context) {}

#[no_mangle]
pub fn lower_el_aarch64_serror_el3(_ctx: *mut Context) {}

// from lower EL (AArch32)
#[no_mangle]
pub fn lower_el_aarch32_sync_el3(_ctx: *mut Context) {}

#[no_mangle]
pub fn lower_el_aarch32_irq_el3(_ctx: *mut Context) {}

#[no_mangle]
pub fn lower_el_aarch32_fiq_el3(_ctx: *mut Context) {}

#[no_mangle]
pub fn lower_el_aarch32_serror_el3(_ctx: *mut Context) {}

//------------------------------------------------------------------------------

pub fn get_esr_el2() -> u32 {
    let esr: u64;
    unsafe { asm!("mrs {}, esr_el2", lateout(reg) esr) };
    esr as u32
}

// from the current EL using the current SP0
#[no_mangle]
pub fn curr_el_sp0_sync_el2(_ctx: *mut Context) {
    driver::uart::puts("EL2 exception: SP0 Sync\n");
}

#[no_mangle]
pub fn curr_el_sp0_irq_el2(_ctx: *mut Context) {
    driver::uart::puts("EL2 exception: SP0 IRQ\n");
}

#[no_mangle]
pub fn curr_el_sp0_fiq_el2(_ctx: *mut Context) {
    driver::uart::puts("EL2 exception: SP0 FIQ\n");
}

#[no_mangle]
pub fn curr_el_sp0_serror_el2(_ctx: *mut Context) {
    driver::uart::puts("EL2 exception: SP0 Error\n");
}

#[no_mangle]
pub fn curr_el_spx_sync_el2(ctx: *mut Context) {
    let r = unsafe { &*ctx };
    driver::uart::puts("EL2 exception: SPX Sync\nELR = 0x");
    driver::uart::hex(r.elr);
    driver::uart::puts("\nSPSR = 0x");
    driver::uart::hex(r.spsr as u64);
    driver::uart::puts("\nESR = 0x");
    driver::uart::hex(get_esr_el2() as u64);
    driver::uart::puts("\n");
}

#[no_mangle]
pub fn curr_el_spx_irq_el2(_ctx: *mut Context) {
    driver::uart::puts("EL2 exception: SPX IRQ\n");
}

#[no_mangle]
pub fn curr_el_spx_fiq_el2(_ctx: *mut Context) {
    driver::uart::puts("EL2 exception: SPX FIQ\n");
}

#[no_mangle]
pub fn curr_el_spx_serror_el2(_ctx: *mut Context) {
    driver::uart::puts("EL2 exception: SPX Error\n");
}

// from lower EL (AArch64)
#[no_mangle]
pub fn lower_el_aarch64_sync_el2(ctx: *mut Context) {
    let r = unsafe { &*ctx };
    driver::uart::puts("EL2 exception: Sync lower AArch64\nELR = ");
    driver::uart::hex(r.elr);
    driver::uart::puts("\nSPSR = 0x");
    driver::uart::hex(r.spsr as u64);
    driver::uart::puts("\nESR = 0x");
    driver::uart::hex(get_esr_el2() as u64);
    driver::uart::puts("\n");
}

#[no_mangle]
pub fn lower_el_aarch64_irq_el2(_ctx: *mut Context) {}

#[no_mangle]
pub fn lower_el_aarch64_fiq_el2(_ctx: *mut Context) {}

#[no_mangle]
pub fn lower_el_aarch64_serror_el2(_ctx: *mut Context) {
    driver::uart::puts("EL2 exception: Error lower AArch64\n");
}

// from lower EL (AArch32)
#[no_mangle]
pub fn lower_el_aarch32_sync_el2(_ctx: *mut Context) {}

#[no_mangle]
pub fn lower_el_aarch32_irq_el2(_ctx: *mut Context) {}

#[no_mangle]
pub fn lower_el_aarch32_fiq_el2(_ctx: *mut Context) {}

#[no_mangle]
pub fn lower_el_aarch32_serror_el2(_ctx: *mut Context) {}

//------------------------------------------------------------------------------

pub fn get_esr_el1() -> u32 {
    let esr: u64;
    unsafe { asm!("mrs {}, esr_el1", lateout(reg) esr) };
    esr as u32
}

// from the current EL using the current SP0
#[no_mangle]
pub fn curr_el_sp0_sync_el1(_ctx: *mut Context) {
    driver::uart::puts("EL1 exception: SP0 Sync\n");
}

#[no_mangle]
pub fn curr_el_sp0_irq_el1(_ctx: *mut Context) {
    driver::uart::puts("EL1 exception: SP0 IRQ\n");
}

#[no_mangle]
pub fn curr_el_sp0_fiq_el1(_ctx: *mut Context) {
    driver::uart::puts("EL1 exception: SP0 FIQ\n");
}

#[no_mangle]
pub fn curr_el_sp0_serror_el1(_ctx: *mut Context) {
    driver::uart::puts("EL1 exception: SP0 Error\n");
}

#[no_mangle]
pub fn curr_el_spx_sync_el1(ctx: *mut Context) {
    let r = unsafe { &*ctx };
    driver::uart::puts("EL1 exception: SPX Sync\nELR = 0x");
    driver::uart::hex(r.elr);
    driver::uart::puts("\nSPSR = 0x");
    driver::uart::hex(r.spsr as u64);
    driver::uart::puts("\nESR = 0x");
    driver::uart::hex(get_esr_el1() as u64);
    driver::uart::puts("\n");
}

#[no_mangle]
pub fn curr_el_spx_irq_el1(_ctx: *mut Context) {
    driver::uart::puts("EL1 exception: SPX IRQ\n");
}

#[no_mangle]
pub fn curr_el_spx_fiq_el1(_ctx: *mut Context) {
    driver::uart::puts("EL1 exception: SPX FIQ\n");
}

#[no_mangle]
pub fn curr_el_spx_serror_el1(_ctx: *mut Context) {
    driver::uart::puts("EL1 exception: SPX Error\n");
}

// from lower EL (AArch64)
#[no_mangle]
pub fn lower_el_aarch64_sync_el1(ctx: *mut Context) {
    let r = unsafe { &*ctx };
    driver::uart::puts("EL1 exception: lower EL Sync\nELR  = 0x");
    driver::uart::hex(r.elr);
    driver::uart::puts("\nSPSR = 0x");
    driver::uart::hex(r.spsr as u64);
    driver::uart::puts("\nESR  = 0x");
    driver::uart::hex(get_esr_el1() as u64);
    driver::uart::puts("\n");
}

#[no_mangle]
pub fn lower_el_aarch64_irq_el1(_ctx: *mut Context) {}

#[no_mangle]
pub fn lower_el_aarch64_fiq_el1(_ctx: *mut Context) {}

#[no_mangle]
pub fn lower_el_aarch64_serror_el1(_ctx: *mut Context) {}

// from lower EL (AArch32)
#[no_mangle]
pub fn lower_el_aarch32_sync_el1(_ctx: *mut Context) {}

#[no_mangle]
pub fn lower_el_aarch32_irq_el1(_ctx: *mut Context) {}

#[no_mangle]
pub fn lower_el_aarch32_fiq_el1(_ctx: *mut Context) {}

#[no_mangle]
pub fn lower_el_aarch32_serror_el1(_ctx: *mut Context) {}
