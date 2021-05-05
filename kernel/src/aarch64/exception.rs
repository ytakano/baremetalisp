use super::context::GpRegs;
use super::cpu;
use super::syscall;
use crate::{driver, paging, print};

const ESR_EL1_EC_MASK: u64 = 0b111111 << 26;
const ESR_EL1_EC_UNKNOWN: u64 = 0b000000 << 26;
const ESR_EL1_EC_WFI_OR_WFE: u64 = 0b000001 << 26;
const ESR_EL1_EC_SVC32: u64 = 0b010001 << 26;
const ESR_EL1_EC_SVC64: u64 = 0b010101 << 26;
const ESR_LE1_EC_DATA: u64 = 0b100100 << 26;
const ESR_LE1_EC_DATA_KERN: u64 = 0b100101 << 26;

//------------------------------------------------------------------------------

// EL2

// from the current EL using the current SP0
#[no_mangle]
pub fn curr_el_sp0_sync_el2(_ctx: *mut GpRegs, _sp: usize) {
    driver::uart::puts("EL2 exception: SP0 Sync\n");
}

#[no_mangle]
pub fn curr_el_sp0_irq_el2(_ctx: *mut GpRegs, _sp: usize) {
    driver::uart::puts("EL2 exception: SP0 IRQ\n");
}

#[no_mangle]
pub fn curr_el_sp0_fiq_el2(_ctx: *mut GpRegs, _sp: usize) {
    driver::uart::puts("EL2 exception: SP0 FIQ\n");
}

#[no_mangle]
pub fn curr_el_sp0_serror_el2(_ctx: *mut GpRegs, _sp: usize) {
    driver::uart::puts("EL2 exception: SP0 Error\n");
}

#[no_mangle]
pub fn curr_el_spx_sync_el2(ctx: *mut GpRegs, _sp: usize) {
    let r = unsafe { &*ctx };
    driver::uart::puts("EL2 exception: SPX Sync\nELR = 0x");
    driver::uart::hex(r.elr);
    driver::uart::puts("\nSPSR = 0x");
    driver::uart::hex(r.spsr as u64);
    driver::uart::puts("\nESR = 0x");
    driver::uart::hex(cpu::esr_el2::get() as u64);
    driver::uart::puts("\n");
}

#[no_mangle]
pub fn curr_el_spx_irq_el2(_ctx: *mut GpRegs, _sp: usize) {
    driver::uart::puts("EL2 exception: SPX IRQ\n");
}

#[no_mangle]
pub fn curr_el_spx_fiq_el2(_ctx: *mut GpRegs, _sp: usize) {
    driver::uart::puts("EL2 exception: SPX FIQ\n");
}

#[no_mangle]
pub fn curr_el_spx_serror_el2(_ctx: *mut GpRegs, _sp: usize) {
    driver::uart::puts("EL2 exception: SPX Error\n");
}

// from lower EL (AArch64)
#[no_mangle]
pub fn lower_el_aarch64_sync_el2(ctx: *mut GpRegs, _sp: usize) {
    let r = unsafe { &*ctx };
    driver::uart::puts("EL2 exception: Sync lower AArch64\nELR = ");
    driver::uart::hex(r.elr);
    driver::uart::puts("\nSPSR = 0x");
    driver::uart::hex(r.spsr as u64);
    driver::uart::puts("\nESR = 0x");
    driver::uart::hex(cpu::esr_el2::get() as u64);
    driver::uart::puts("\n");
}

#[no_mangle]
pub fn lower_el_aarch64_irq_el2(_ctx: *mut GpRegs, _sp: usize) {
    driver::uart::puts("EL2 exception: IRQ\n");
}

#[no_mangle]
pub fn lower_el_aarch64_fiq_el2(_ctx: *mut GpRegs, _sp: usize) {
    driver::uart::puts("EL2 exception: FIQ\n");
}

#[no_mangle]
pub fn lower_el_aarch64_serror_el2(_ctx: *mut GpRegs, _sp: usize) {
    driver::uart::puts("EL2 exception: Error lower AArch64\n");
}

// from lower EL (AArch32)
#[no_mangle]
pub fn lower_el_aarch32_sync_el2(_ctx: *mut GpRegs, _sp: usize) {}

#[no_mangle]
pub fn lower_el_aarch32_irq_el2(_ctx: *mut GpRegs, _sp: usize) {}

#[no_mangle]
pub fn lower_el_aarch32_fiq_el2(_ctx: *mut GpRegs, _sp: usize) {}

#[no_mangle]
pub fn lower_el_aarch32_serror_el2(_ctx: *mut GpRegs, _sp: usize) {}

//------------------------------------------------------------------------------

// EL1

// from the current EL using the SP_EL0
#[no_mangle]
pub fn curr_el_sp0_sync_el1(ctx: *mut GpRegs, sp: usize) {
    driver::uart::puts("EL1 exception: SP0 Sync\n");
    sync_el1(ctx, sp);
}

#[no_mangle]
pub fn curr_el_sp0_irq_el1(_ctx: *mut GpRegs, _sp: usize) {
    driver::uart::puts("EL1 exception: SP0 IRQ\n");
}

#[no_mangle]
pub fn curr_el_sp0_fiq_el1(_ctx: *mut GpRegs, _sp: usize) {
    driver::uart::puts("EL1 exception: SP0 FIQ\n");
}

#[no_mangle]
pub fn curr_el_sp0_serror_el1(_ctx: *mut GpRegs, _sp: usize) {
    driver::uart::puts("EL1 exception: SP0 Error\n");
}

// from the current EL using the SP_EL1
#[no_mangle]
pub fn curr_el_spx_sync_el1(ctx: *mut GpRegs, _sp: usize) {
    let r = unsafe { &*ctx };
    driver::uart::puts("EL1 exception: SPX Sync\nELR = 0x");
    driver::uart::hex(r.elr);
    driver::uart::puts("\nSPSR = 0x");
    driver::uart::hex(r.spsr as u64);
    driver::uart::puts("\nESR = 0x");
    driver::uart::hex(cpu::esr_el1::get() as u64);
    driver::uart::puts("\n");
    driver::delays::forever();
}

#[no_mangle]
pub fn curr_el_spx_irq_el1(_ctx: *mut GpRegs, _sp: usize) {
    driver::uart::puts("EL1 exception: SPX IRQ\n");
}

#[no_mangle]
pub fn curr_el_spx_fiq_el1(_ctx: *mut GpRegs, _sp: usize) {
    driver::uart::puts("EL1 exception: SPX FIQ\n");
}

#[no_mangle]
pub fn curr_el_spx_serror_el1(_ctx: *mut GpRegs, _sp: usize) {
    driver::uart::puts("EL1 exception: SPX Error\n");
}

// from lower EL (AArch64)
#[no_mangle]
pub fn lower_el_aarch64_sync_el1(ctx: *mut GpRegs, sp: usize) {
    sync_el1(ctx, sp);
}

fn sync_el1(ctx: *mut GpRegs, _sp: usize) {
    let r = unsafe { &mut *ctx };
    let esr = cpu::esr_el1::get();

    let ec = esr & ESR_EL1_EC_MASK;
    match ec {
        ESR_EL1_EC_WFI_OR_WFE => print::msg("EL1 Exception", "WFI or WFE"),
        ESR_LE1_EC_DATA => print::msg("EL1 Exception", "DATA"),
        ESR_LE1_EC_DATA_KERN => {
            let far_el1 = cpu::far_el1::get();
            print::msg("EL1 Exception", "DATA Abort (Kernel)");
            print::hex64("ELR", r.elr);
            print::hex64("FAR_EL1", far_el1);
            paging::map(far_el1 as usize, true);
        }
        ESR_EL1_EC_SVC64 => {
            let n = syscall::handle64(r);
            r.x0 = n as u64;
        }
        _ => {
            driver::uart::puts("ELR = ");
            driver::uart::hex(r.elr);
            driver::uart::puts("\nSPSR = 0x");
            driver::uart::hex(r.spsr as u64);
            driver::uart::puts("\nESR = 0x");
            driver::uart::hex(esr);
            driver::uart::puts("\nEC = 0b");
            driver::uart::bin8((ec >> 26) as u8);
            driver::uart::puts("\n");
            print::msg("EL1 Exception", "unknown")
        }
    }
}

#[no_mangle]
pub fn lower_el_aarch64_irq_el1(_ctx: *mut GpRegs, _sp: usize) {
    driver::uart::puts("EL1 exception: IRQ\n");
}

#[no_mangle]
pub fn lower_el_aarch64_fiq_el1(_ctx: *mut GpRegs, _sp: usize) {
    driver::uart::puts("EL1 exception: FIQ\n");
}

#[no_mangle]
pub fn lower_el_aarch64_serror_el1(_ctx: *mut GpRegs, _sp: usize) {
    driver::uart::puts("EL1 exception: Error\n");
}

// from lower EL (AArch32)
#[no_mangle]
pub fn lower_el_aarch32_sync_el1(_ctx: *mut GpRegs, _sp: usize) {}

#[no_mangle]
pub fn lower_el_aarch32_irq_el1(_ctx: *mut GpRegs, _sp: usize) {}

#[no_mangle]
pub fn lower_el_aarch32_fiq_el1(_ctx: *mut GpRegs, _sp: usize) {}

#[no_mangle]
pub fn lower_el_aarch32_serror_el1(_ctx: *mut GpRegs, _sp: usize) {}
