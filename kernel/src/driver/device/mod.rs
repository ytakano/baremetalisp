// Raspberry Pi 4, Broadcom BCM2711
#[cfg(any(feature = "raspi3", feature = "raspi4"))]
pub mod bcm2711;

// Pine64, Allwineer A64
#[cfg(feature = "pine64")]
pub mod a64;
