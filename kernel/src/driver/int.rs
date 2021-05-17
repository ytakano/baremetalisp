#[cfg(feature = "pine64")]
pub type DevIRQManger = super::gic::IRQManager;

#[cfg(any(feature = "raspi3", feature = "raspi4"))]
pub type DevIRQManger = super::device::raspi::int::IRQManager;

impl DevIRQManger {
    pub const fn new() -> Self {
        DevIRQManger::new_mng()
    }
}
