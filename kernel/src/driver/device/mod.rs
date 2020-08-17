// Raspberry Pi 4, Broadcom BCM2xxx
#[cfg(any(feature = "raspi3", feature = "raspi4"))]
pub mod raspi;

// Pine64, Allwineer sunxi
#[cfg(feature = "pine64")]
pub mod allwinner;

pub mod generic;
