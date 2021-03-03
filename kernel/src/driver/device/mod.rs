// Raspberry Pi 4, Broadcom BCM2xxx
#[cfg(any(feature = "raspi3", feature = "raspi4"))]
pub(crate) mod raspi;

// Pine64, Allwineer sunxi
#[cfg(feature = "pine64")]
pub(crate) mod allwinner;

pub(crate) mod generic;
