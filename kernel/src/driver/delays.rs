pub(super) trait Delays {
    fn init();
    fn get_timer_value() -> usize;
    fn wait_microsec(usec: usize);
}

#[cfg(any(feature = "raspi3", feature = "pine64"))]
type DevDelays = super::device::generic::delays::Delays;

#[cfg(feature = "raspi4")]
type DevDelays = super::device::raspi::delays::Delays;

impl DevDelays where DevDelays: Delays {}

pub fn init() {
    DevDelays::init();
}

pub fn get_timer_value() -> usize {
    DevDelays::get_timer_value()
}

/// wait microsec
pub fn wait_microsec(usec: usize) {
    DevDelays::wait_microsec(usec);
}

/// wait milisec
pub fn wait_milisec(msec: usize) {
    wait_microsec(msec * 1000);
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
