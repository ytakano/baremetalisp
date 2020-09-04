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

    const SMC64_STD_SERVICE: u64 = 0xc4;
    const SMC32_STD_SERVICE: u64 = 0x84;

    pub const SMC_TO_NORMAL: u64 = 0xc400F001;
    pub const SMC_TO_SECURE: u64 = 0xc400F002;

    /// switch to normal world
    #[inline(never)]
    pub fn to_normal() {
        unsafe {
            asm!(
                "mov x0, {}
                 smc #0",
                 in(reg) SMC_TO_NORMAL
            )
        }
    }

    /// switch to secure world
    #[inline(never)]
    pub fn to_secure() {
        unsafe {
            asm!(
                "mov x0, {}
                 smc #0",
                 in(reg) SMC_TO_SECURE
            )
        }
    }

    pub fn handler(ctx: &context::GpRegs, sp: usize) {
        uart::puts("SMC 0x");
        uart::hex32(ctx.x0 as u32);
        uart::puts("\n");

        let id = (ctx.x0 >> 24) & 0xff;

        if id == SMC64_STD_SERVICE || id == SMC32_STD_SERVICE {
            el3::smc_std_service(ctx, sp);
        }
    }
}
