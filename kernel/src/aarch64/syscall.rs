// super visor call (from EL0 to EL1)
pub mod svc {
    use crate::aarch64::context;
    use crate::driver::uart;
    use crate::el1;

    pub const SYS_SWITCH_WORLD: u64 = 1;

    /// switch to normal mode
    pub fn switch_world() {
        unsafe { asm!("svc #1") }
    }

    pub fn handle64(id: u64, _ctx: &context::GpRegs, _sp: usize) {
        uart::puts("Sycall #");
        uart::decimal(id);
        uart::puts("\n");

        match id {
            SYS_SWITCH_WORLD => el1::sys_switch(),
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
    pub const SMC_TO_SECURE: u64 = 2;

    /// switch to normal world
    pub fn to_normal() {
        unsafe { asm!("smc #1") }
    }

    /// switch to secure world
    pub fn to_secure() {
        unsafe { asm!("smc #2") }
    }

    pub fn handle64(id: u64, ctx: &context::GpRegs, sp: usize) {
        uart::puts("SMC #");
        uart::decimal(id);
        uart::puts("\n");

        match id {
            SMC_TO_NORMAL => el3::switch_world(ctx, sp, false),
            SMC_TO_SECURE => el3::switch_world(ctx, sp, true),
            _ => (),
        }
    }
}
