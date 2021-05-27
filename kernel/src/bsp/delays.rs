use crate::driver::delays::{self, Delays};

//-------------------------------------------------------------------------
// Raspberry Pi 3 (Qemu) or Pine64
#[cfg(any(feature = "raspi3", feature = "pine64"))]
type DevDelays = delays::generic::Delays;

#[cfg(any(feature = "raspi3", feature = "pine64"))]
const BSP_DELAYS: DevDelays = DevDelays::new();
//-------------------------------------------------------------------------
// Raspberry Pi 4
#[cfg(feature = "raspi4")]
type DevDelays = delays::bcm2xxx::Delays;

#[cfg(feature = "raspi4")]
const BSP_DELAYS: delays::bcm2xxx::Delays =
    delays::bcm2xxx::Delays::new(super::raspi::memory::MMIO_BASE + 0x3000);
//-------------------------------------------------------------------------

impl DevDelays where DevDelays: Delays {}

pub fn init() {
    BSP_DELAYS.init();
}

pub fn get_timer_value() -> usize {
    BSP_DELAYS.get_timer_value()
}

/// wait microsec
pub fn wait_microsec(usec: usize) {
    BSP_DELAYS.wait_microsec(usec);
}

/// wait milisec
pub fn wait_milisec(msec: usize) {
    BSP_DELAYS.wait_milisec(msec);
}

/// Wait N CPU cycles
pub fn wait_cycles(n: usize) {
    if n > 0 {
        for _ in 0..n {
            unsafe { asm!("nop;") };
        }
    }
}

pub fn forever() -> ! {
    loop {
        unsafe { asm!("wfe") };
    }
}
