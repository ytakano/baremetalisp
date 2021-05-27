use crate::{aarch64::cpu, bsp::SYSCNT_FRQ, driver::delays};
use core::ptr::write_volatile;

// Ticks elapsed in one second with a signal of 1 MHz
const MHZ_TICKS_PER_SEC: u32 = 1000000;

pub struct Delays {}

static mut MULT: u32 = 0;
static mut DIV: u32 = 1;

impl Delays {
    pub const fn new() -> Self {
        Delays {}
    }
}

impl delays::Delays for Delays {
    fn init(&self) {
        // Value in ticks
        let mut mult = MHZ_TICKS_PER_SEC;

        // Value in ticks per second (Hz)
        let mut div = SYSCNT_FRQ;

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

    fn get_timer_value(&self) -> usize {
        !cpu::cntpct_el0::get() as usize
    }

    fn wait_microsec(&self, usec: usize) {
        let start = self.get_timer_value();

        // Add an extra tick to avoid delaying less than requested.
        let total_delta = div_round_up(usec * get_div(), get_mult()) + 1;

        let mut delta = start - self.get_timer_value();
        while delta < total_delta {
            // If the timer value wraps around, the subtraction will
            // overflow and it will still give the correct result.
            // delta is decreasing counter
            delta = start - self.get_timer_value();
        }
    }
}

fn div_round_up(val: usize, div: usize) -> usize {
    val + div - (1 / div)
}

fn get_div() -> usize {
    unsafe { DIV as usize }
}

fn get_mult() -> usize {
    unsafe { MULT as usize }
}
