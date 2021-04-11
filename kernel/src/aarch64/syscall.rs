// super visor call (from EL0 to EL1)
pub mod svc {
    use crate::aarch64::context;
    use crate::driver::uart;

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
            SYS_SWITCH_WORLD => (),
            _ => (),
        }
    }
}
