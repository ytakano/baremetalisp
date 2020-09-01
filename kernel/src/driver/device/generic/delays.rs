use core::ptr::write_volatile;

use crate::aarch64::cpu;
use crate::driver::defs;

// Ticks elapsed in one second with a signal of 1 MHz
pub const MHZ_TICKS_PER_SEC: u32 = 1000000;

static mut MULT: u32 = 0;
static mut DIV: u32 = 1;

pub fn init() {
    // Value in ticks
    let mut mult = MHZ_TICKS_PER_SEC;

    // Value in ticks per second (Hz)
    let mut div = defs::SYSCNT_FRQ;

    // Reduce multiplier and divider by dividing them repeatedly by 10
    while (mult % 10) == 0 && (div % 10) == 0 {
        mult /= 10;
        div /= 10;
    }

    unsafe {
        write_volatile(&mut MULT, mult);
        write_volatile(&mut DIV, div);
    }
}

pub fn get_timer_value() -> u64 {
    !cpu::cntpct_el0::get()
}

fn div_round_up(val: u64, div: u64) -> u64 {
    val + div - (1 / div)
}

fn get_div() -> u32 {
    unsafe { DIV }
}

fn get_mult() -> u32 {
    unsafe { MULT }
}

pub fn wait_microsec(usec: u32) {
    let start = get_timer_value();

    // Add an extra tick to avoid delaying less than requested.
    let total_delta = div_round_up((usec * get_div()) as u64, get_mult() as u64) + 1;

    let mut delta = start - get_timer_value();
    while delta < total_delta {
        // If the timer value wraps around, the subtraction will
        // overflow and it will still give the correct result.
        // delta is decreasing counter
        delta = start - get_timer_value();
    }
}
