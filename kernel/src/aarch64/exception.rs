use super::context::GpRegs;
use crate::driver;
use crate::el3;

//------------------------------------------------------------------------------

pub fn get_esr_el3() -> u32 {
    let esr: u64;
    unsafe { asm!("mrs {}, esr_el3", lateout(reg) esr) };
    esr as u32
}

// from the current EL using the current SP0
#[no_mangle]
pub fn curr_el_sp0_sync_el3(__ctx: *mut GpRegs) {
    driver::uart::puts("EL3 exception: SP0 Sync\n");
}

#[no_mangle]
pub fn curr_el_sp0_irq_el3(__ctx: *mut GpRegs) {
    driver::uart::puts("EL3 exception: SP0 IRQ\n");
}

#[no_mangle]
pub fn curr_el_sp0_fiq_el3(__ctx: *mut GpRegs) {
    driver::uart::puts("EL3 exception: SP0 FIQ\n");
}

#[no_mangle]
pub fn curr_el_sp0_serror_el3(__ctx: *mut GpRegs) {
    driver::uart::puts("EL3 exception: SP0 Error\n");
}

// from the current EL using the current SP
#[no_mangle]
pub fn curr_el_spx_sync_el3(ctx: *mut GpRegs) {
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
pub fn curr_el_spx_irq_el3(_ctx: *mut GpRegs) {
    driver::uart::puts("EL3 exception: SPX IRQ\n");
}

#[no_mangle]
pub fn curr_el_spx_fiq_el3(_ctx: *mut GpRegs) {
    driver::uart::puts("EL3 exception: SPX FIQ\n");
}

#[no_mangle]
pub fn curr_el_spx_serror_el3(ctx: *mut GpRegs) {
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
pub fn lower_el_aarch64_sync_el3(ctx: *mut GpRegs) {
    // secure monitor call
    el3::handle_smc64(unsafe { &*ctx });
}

#[no_mangle]
pub fn lower_el_aarch64_irq_el3(_ctx: *mut GpRegs) {}

#[no_mangle]
pub fn lower_el_aarch64_fiq_el3(_ctx: *mut GpRegs) {}

#[no_mangle]
pub fn lower_el_aarch64_serror_el3(_ctx: *mut GpRegs) {}

// from lower EL (AArch32)
#[no_mangle]
pub fn lower_el_aarch32_sync_el3(_ctx: *mut GpRegs) {}

#[no_mangle]
pub fn lower_el_aarch32_irq_el3(_ctx: *mut GpRegs) {}

#[no_mangle]
pub fn lower_el_aarch32_fiq_el3(_ctx: *mut GpRegs) {}

#[no_mangle]
pub fn lower_el_aarch32_serror_el3(_ctx: *mut GpRegs) {}

//------------------------------------------------------------------------------

pub fn get_esr_el2() -> u32 {
    let esr: u64;
    unsafe { asm!("mrs {}, esr_el2", lateout(reg) esr) };
    esr as u32
}

// from the current EL using the current SP0
#[no_mangle]
pub fn curr_el_sp0_sync_el2(_ctx: *mut GpRegs) {
    driver::uart::puts("EL2 exception: SP0 Sync\n");
}

#[no_mangle]
pub fn curr_el_sp0_irq_el2(_ctx: *mut GpRegs) {
    driver::uart::puts("EL2 exception: SP0 IRQ\n");
}

#[no_mangle]
pub fn curr_el_sp0_fiq_el2(_ctx: *mut GpRegs) {
    driver::uart::puts("EL2 exception: SP0 FIQ\n");
}

#[no_mangle]
pub fn curr_el_sp0_serror_el2(_ctx: *mut GpRegs) {
    driver::uart::puts("EL2 exception: SP0 Error\n");
}

#[no_mangle]
pub fn curr_el_spx_sync_el2(ctx: *mut GpRegs) {
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
pub fn curr_el_spx_irq_el2(_ctx: *mut GpRegs) {
    driver::uart::puts("EL2 exception: SPX IRQ\n");
}

#[no_mangle]
pub fn curr_el_spx_fiq_el2(_ctx: *mut GpRegs) {
    driver::uart::puts("EL2 exception: SPX FIQ\n");
}

#[no_mangle]
pub fn curr_el_spx_serror_el2(_ctx: *mut GpRegs) {
    driver::uart::puts("EL2 exception: SPX Error\n");
}

// from lower EL (AArch64)
#[no_mangle]
pub fn lower_el_aarch64_sync_el2(ctx: *mut GpRegs) {
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
pub fn lower_el_aarch64_irq_el2(_ctx: *mut GpRegs) {}

#[no_mangle]
pub fn lower_el_aarch64_fiq_el2(_ctx: *mut GpRegs) {}

#[no_mangle]
pub fn lower_el_aarch64_serror_el2(_ctx: *mut GpRegs) {
    driver::uart::puts("EL2 exception: Error lower AArch64\n");
}

// from lower EL (AArch32)
#[no_mangle]
pub fn lower_el_aarch32_sync_el2(_ctx: *mut GpRegs) {}

#[no_mangle]
pub fn lower_el_aarch32_irq_el2(_ctx: *mut GpRegs) {}

#[no_mangle]
pub fn lower_el_aarch32_fiq_el2(_ctx: *mut GpRegs) {}

#[no_mangle]
pub fn lower_el_aarch32_serror_el2(_ctx: *mut GpRegs) {}

//------------------------------------------------------------------------------

pub fn get_esr_el1() -> u32 {
    let esr: u64;
    unsafe { asm!("mrs {}, esr_el1", lateout(reg) esr) };
    esr as u32
}

// from the current EL using the current SP0
#[no_mangle]
pub fn curr_el_sp0_sync_el1(_ctx: *mut GpRegs) {
    driver::uart::puts("EL1 exception: SP0 Sync\n");
}

#[no_mangle]
pub fn curr_el_sp0_irq_el1(_ctx: *mut GpRegs) {
    driver::uart::puts("EL1 exception: SP0 IRQ\n");
}

#[no_mangle]
pub fn curr_el_sp0_fiq_el1(_ctx: *mut GpRegs) {
    driver::uart::puts("EL1 exception: SP0 FIQ\n");
}

#[no_mangle]
pub fn curr_el_sp0_serror_el1(_ctx: *mut GpRegs) {
    driver::uart::puts("EL1 exception: SP0 Error\n");
}

#[no_mangle]
pub fn curr_el_spx_sync_el1(ctx: *mut GpRegs) {
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
pub fn curr_el_spx_irq_el1(_ctx: *mut GpRegs) {
    driver::uart::puts("EL1 exception: SPX IRQ\n");
}

#[no_mangle]
pub fn curr_el_spx_fiq_el1(_ctx: *mut GpRegs) {
    driver::uart::puts("EL1 exception: SPX FIQ\n");
}

#[no_mangle]
pub fn curr_el_spx_serror_el1(_ctx: *mut GpRegs) {
    driver::uart::puts("EL1 exception: SPX Error\n");
}

// from lower EL (AArch64)
#[no_mangle]
pub fn lower_el_aarch64_sync_el1(ctx: *mut GpRegs) {
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
pub fn lower_el_aarch64_irq_el1(_ctx: *mut GpRegs) {}

#[no_mangle]
pub fn lower_el_aarch64_fiq_el1(_ctx: *mut GpRegs) {}

#[no_mangle]
pub fn lower_el_aarch64_serror_el1(_ctx: *mut GpRegs) {}

// from lower EL (AArch32)
#[no_mangle]
pub fn lower_el_aarch32_sync_el1(_ctx: *mut GpRegs) {}

#[no_mangle]
pub fn lower_el_aarch32_irq_el1(_ctx: *mut GpRegs) {}

#[no_mangle]
pub fn lower_el_aarch32_fiq_el1(_ctx: *mut GpRegs) {}

#[no_mangle]
pub fn lower_el_aarch32_serror_el1(_ctx: *mut GpRegs) {}
