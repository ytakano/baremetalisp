// super visor call (from EL0 to EL1)
pub mod svc {
    use crate::aarch64::context;
    use crate::driver::uart;
    use crate::el1;

    pub const SYS_SWITCH: u64 = 1;

    /// switch to normal mode
    pub fn switch() {
        unsafe { asm!("svc #1") }
    }

    pub fn handle64(id: u64, _ctx: &context::GpRegs) {
        uart::puts("received sycall #");
        uart::decimal(id);
        uart::puts("\n");

        match id {
            SYS_SWITCH => el1::sys_switch(),
            _ => (),
        }
    }
}

//-----------------------------------------------------------------------------

// secure monitor call (from EL1 to EL3)
pub mod smc {
    use crate::aarch64::context;
    use crate::driver::uart;
    use crate::el3;

    pub const SMC_TO_NORMAL: u64 = 1;

    /// switch to normal mode
    pub fn to_normal() {
        unsafe { asm!("smc #1") }
    }

    pub fn handle64(id: u64, ctx: &context::GpRegs) {
        uart::puts("received SMC #");
        uart::decimal(id);
        uart::puts("\n");

        match id {
            SMC_TO_NORMAL => el3::smc_to_normal(ctx),
            _ => (),
        }
    }
}
