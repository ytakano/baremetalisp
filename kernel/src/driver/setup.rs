#[cfg(any(feature = "raspi3", feature = "raspi4"))]
use super::device::raspi::setup;

#[cfg(feature = "pine64")]
use super::device::allwinner::setup;

pub fn platform_setup() {
    setup::platform_setup();
}
