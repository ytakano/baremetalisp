#[cfg(any(feature = "raspi3", feature = "pine64"))]
use super::device::generic::delays;

#[cfg(feature = "raspi4")]
use super::device::raspi::delays;

pub fn init() {
    delays::init();
}

pub fn get_timer_value() -> u64 {
    delays::get_timer_value()
}

/// wait microsec
pub fn wait_microsec(usec: u32) {
    delays::wait_microsec(usec);
}

/// wait milisec
pub fn wait_milisec(msec: u32) {
    wait_microsec(msec * 1000);
}

/// Wait N CPU cycles
pub fn wait_cycles(n: u32) {
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
