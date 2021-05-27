// Raspberry Pi 4, Broadcom BCM2xxx
#[cfg(any(feature = "raspi3", feature = "raspi4"))]
mod raspi;

#[cfg(any(feature = "raspi3", feature = "raspi4"))]
use raspi as board;
//---------------------------------------------------
// Pine64, Allwineer sunxi
#[cfg(feature = "pine64")]
mod allwinner;

#[cfg(feature = "pine64")]
use allwinner as board;
//---------------------------------------------------

pub const SYSCNT_FRQ: u32 = board::SYSCNT_FRQ;
type BoardInit = board::Init; // board dependent initializer

pub mod delays;
pub mod int;
pub mod memory;
pub mod uart;

pub trait BSPInit {
    fn early_init();
    fn init();
}

impl BoardInit where BoardInit: BSPInit {}

pub fn early_init() {
    delays::init();
    // TODO: init uart
    BoardInit::early_init();
}

pub fn init() {
    int::init();
    BoardInit::init();
}
