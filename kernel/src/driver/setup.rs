pub(super) trait Setup {
    fn platform_setup();
    fn early_platform_setup();
}

#[cfg(feature = "pine64")]
type DevSetup = super::device::allwinner::setup::Setup;

#[cfg(any(feature = "raspi3", feature = "raspi4"))]
type DevSetup = super::device::raspi::setup::Setup;

impl DevSetup where DevSetup: Setup {}

/// Initialize devices
pub fn platform_setup() {
    DevSetup::platform_setup();
}

pub fn early_platform_setup() {
    DevSetup::early_platform_setup();
}
