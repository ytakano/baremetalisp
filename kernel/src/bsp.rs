// Raspberry Pi 4, Broadcom BCM2xxx
#[cfg(any(feature = "raspi3", feature = "raspi4"))]
mod raspi;

// Pine64, Allwineer sunxi
#[cfg(feature = "pine64")]
mod allwinner;

pub mod int;
pub mod memory;
