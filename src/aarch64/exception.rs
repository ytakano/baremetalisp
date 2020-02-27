use crate::driver;

// from the current EL using the current SP0
#[no_mangle]
pub fn curr_el_sp0_sync_el2() {
    driver::uart::puts("exception: SP0 Sync\n");
}

#[no_mangle]
pub fn curr_el_sp0_irq_el2() {
    driver::uart::puts("exception: SP0 IRQ\n");
}

#[no_mangle]
pub fn curr_el_sp0_fiq_el2() {
    driver::uart::puts("exception: SP0 FIQ\n");
}

#[no_mangle]
pub fn curr_el_sp0_serror_el2() {
    driver::uart::puts("exception: SP0 Error\n");
}

// from the current EL using the current SP
#[no_mangle]
pub fn curr_el_spx_sync_el2() {
    driver::uart::puts("exception: SPX Sync\n");
}

#[no_mangle]
pub fn curr_el_spx_irq_el2() {
    driver::uart::puts("exception: SPX IRQ\n");
}

#[no_mangle]
pub fn curr_el_spx_fiq_el2() {
    driver::uart::puts("exception: SPX FIQ\n");
}

#[no_mangle]
pub fn curr_el_spx_serror_el2() {
    driver::uart::puts("exception: SPX Error\n");
}

// from lower EL (AArch64)
#[no_mangle]
pub fn lower_el_aarch64_sync_el2() {

}

#[no_mangle]
pub fn lower_el_aarch64_irq_el2() {

}

#[no_mangle]
pub fn lower_el_aarch64_fiq_el2() {

}

#[no_mangle]
pub fn lower_el_aarch64_serror_el2() {

}

// from lower EL (AArch32)
#[no_mangle]
pub fn lower_el_aarch32_sync_el2() {

}

#[no_mangle]
pub fn lower_el_aarch32_irq_el2() {

}

#[no_mangle]
pub fn lower_el_aarch32_fiq_el2() {

}

#[no_mangle]
pub fn lower_el_aarch32_serror_el2() {

}