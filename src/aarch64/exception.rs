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
    esr: u64,  // exception syndrome register
    spsr: u32, // saved program status register
    _unused: [u8; 12]
}

// from the current EL using the current SP0
#[no_mangle]
pub fn curr_el_sp0_sync_el3(ctx: *mut Context) {
    driver::uart::puts("exception: SP0 Sync\n");
}

#[no_mangle]
pub fn curr_el_sp0_irq_el3(ctx: *mut Context) {
    driver::uart::puts("exception: SP0 IRQ\n");
}

#[no_mangle]
pub fn curr_el_sp0_fiq_el3(ctx: *mut Context) {
    driver::uart::puts("exception: SP0 FIQ\n");
}

#[no_mangle]
pub fn curr_el_sp0_serror_el3(ctx: *mut Context) {
    driver::uart::puts("exception: SP0 Error\n");
}

// from the current EL using the current SP
#[no_mangle]
pub fn curr_el_spx_sync_el3(ctx: *mut Context) {
    let r = unsafe { &*ctx };
    driver::uart::puts("exception: SPX Sync\nESR = 0x");
    driver::uart::hex(r.esr);
    driver::uart::puts("\nSPSR = 0x");
    driver::uart::hex(r.spsr as u64);
    driver::uart::puts("\n");
}

#[no_mangle]
pub fn curr_el_spx_irq_el3(ctx: *mut Context) {
    driver::uart::puts("exception: SPX IRQ\n");
}

#[no_mangle]
pub fn curr_el_spx_fiq_el3(ctx: *mut Context) {
    driver::uart::puts("exception: SPX FIQ\n");
}

#[no_mangle]
pub fn curr_el_spx_serror_el3(ctx: *mut Context) {
    driver::uart::puts("exception: SPX Error\n");
}

// from lower EL (AArch64)
#[no_mangle]
pub fn lower_el_aarch64_sync_el3(ctx: *mut Context) {

}

#[no_mangle]
pub fn lower_el_aarch64_irq_el3(ctx: *mut Context) {

}

#[no_mangle]
pub fn lower_el_aarch64_fiq_el3(ctx: *mut Context) {

}

#[no_mangle]
pub fn lower_el_aarch64_serror_el3(ctx: *mut Context) {

}

// from lower EL (AArch32)
#[no_mangle]
pub fn lower_el_aarch32_sync_el3(ctx: *mut Context) {

}

#[no_mangle]
pub fn lower_el_aarch32_irq_el3(ctx: *mut Context) {

}

#[no_mangle]
pub fn lower_el_aarch32_fiq_el3(ctx: *mut Context) {

}

#[no_mangle]
pub fn lower_el_aarch32_serror_el3(ctx: *mut Context) {

}