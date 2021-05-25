// Raspberry Pi 4, Broadcom BCM2xxx
#[cfg(any(feature = "raspi3", feature = "raspi4"))]
mod raspi;

// Pine64, Allwineer sunxi
#[cfg(feature = "pine64")]
mod allwinner;

pub mod int;
pub mod memory;
pub mod uart;

pub trait BSPInit {
    fn early_init();
    fn init();
}

#[cfg(feature = "pine64")]
type DevInit = allwinner::Init;

#[cfg(any(feature = "raspi3", feature = "raspi4"))]
type DevInit = raspi::Init;

impl DevInit where DevInit: BSPInit {}

pub fn early_init() {
    DevInit::early_init();
}

pub fn init() {
    int::init();
    DevInit::init();
}
