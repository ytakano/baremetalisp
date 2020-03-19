use crate::driver;

#[repr(C)]
pub struct Context {
     x0: u64,
     x1: u64,
     x2: u64,
     x3: u64,
     x4: u64,
     x5: u64,
     x6: u64,
     x7: u64,
     x8: u64,
     x9: u64,
    x10: u64,
    x11: u64,
    x12: u64,
    x13: u64,
    x14: u64,
    x15: u64,
    x16: u64,
    x17: u64,
    x18: u64,
    x19: u64,
    x20: u64,
    x21: u64,
    x22: u64,
    x23: u64,
    x24: u64,
    x25: u64,
    x26: u64,
    x27: u64,
    x28: u64,
    x29: u64,
    x30: u64,  // link register
    elr: u64,  // exception link register
    spsr: u32, // saved program status register
    _unused: [u8; 12]
}

//------------------------------------------------------------------------------

pub fn get_esr_el3() -> u32 {
    let esr;
    unsafe { asm!("mrs $0, esr_el3" : "=r"(esr)) };
    esr
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
pub fn lower_el_aarch64_sync_el3(_ctx: *mut Context) {

}

#[no_mangle]
pub fn lower_el_aarch64_irq_el3(_ctx: *mut Context) {

}

#[no_mangle]
pub fn lower_el_aarch64_fiq_el3(_ctx: *mut Context) {

}

#[no_mangle]
pub fn lower_el_aarch64_serror_el3(_ctx: *mut Context) {

}

// from lower EL (AArch32)
#[no_mangle]
pub fn lower_el_aarch32_sync_el3(_ctx: *mut Context) {

}

#[no_mangle]
pub fn lower_el_aarch32_irq_el3(_ctx: *mut Context) {

}

#[no_mangle]
pub fn lower_el_aarch32_fiq_el3(_ctx: *mut Context) {

}

#[no_mangle]
pub fn lower_el_aarch32_serror_el3(_ctx: *mut Context) {

}

//------------------------------------------------------------------------------

pub fn get_esr_el2() -> u32 {
    let esr;
    unsafe { asm!("mrs $0, esr_el2" : "=r"(esr)) };
    esr
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
pub fn lower_el_aarch64_irq_el2(_ctx: *mut Context) {

}

#[no_mangle]
pub fn lower_el_aarch64_fiq_el2(_ctx: *mut Context) {

}

#[no_mangle]
pub fn lower_el_aarch64_serror_el2(_ctx: *mut Context) {
    driver::uart::puts("EL2 exception: Error lower AArch64\n");
}

// from lower EL (AArch32)
#[no_mangle]
pub fn lower_el_aarch32_sync_el2(_ctx: *mut Context) {

}

#[no_mangle]
pub fn lower_el_aarch32_irq_el2(_ctx: *mut Context) {

}

#[no_mangle]
pub fn lower_el_aarch32_fiq_el2(_ctx: *mut Context) {

}

#[no_mangle]
pub fn lower_el_aarch32_serror_el2(_ctx: *mut Context) {

}

//------------------------------------------------------------------------------

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
pub fn lower_el_aarch64_sync_el1(_ctx: *mut Context) {

}

#[no_mangle]
pub fn lower_el_aarch64_irq_el1(_ctx: *mut Context) {

}

#[no_mangle]
pub fn lower_el_aarch64_fiq_el1(_ctx: *mut Context) {

}

#[no_mangle]
pub fn lower_el_aarch64_serror_el1(_ctx: *mut Context) {

}

// from lower EL (AArch32)
#[no_mangle]
pub fn lower_el_aarch32_sync_el1(_ctx: *mut Context) {

}

#[no_mangle]
pub fn lower_el_aarch32_irq_el1(_ctx: *mut Context) {

}

#[no_mangle]
pub fn lower_el_aarch32_fiq_el1(_ctx: *mut Context) {

}

#[no_mangle]
pub fn lower_el_aarch32_serror_el1(_ctx: *mut Context) {

}