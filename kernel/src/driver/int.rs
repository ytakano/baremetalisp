#[cfg(feature = "pine64")]
pub type DevIRQManger = super::gic::IRQManager;

#[cfg(any(feature = "raspi3", feature = "raspi4"))]
pub type DevIRQManger = super::device::raspi::int::IRQManager;

#[cfg(any(feature = "raspi3", feature = "raspi4"))]
pub type DevIRQNumber = super::device::raspi::int::IRQNumber;
