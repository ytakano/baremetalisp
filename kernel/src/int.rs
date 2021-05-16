use crate::aarch64;

pub trait InterMask {
    fn new() -> Self;
    fn enable_irq();
    fn unmask(self);
}

pub type ArchIntMask = aarch64::int::AA64Mask;

impl ArchIntMask where ArchIntMask: InterMask {}

pub fn mask() -> ArchIntMask {
    ArchIntMask::new()
}

pub fn enable_irq() {
    ArchIntMask::enable_irq();
}
