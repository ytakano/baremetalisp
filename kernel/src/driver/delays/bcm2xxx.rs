//! BCM2xxxx System Timer
//! See page 172 of
//! https://www.raspberrypi.org/app/uploads/2012/02/BCM2835-ARM-Peripherals.pdf

use crate::{driver::delays, mmio_rw_base};

pub struct Delays {
    base: usize,
}

impl Delays {
    pub const fn new(base: usize) -> Self {
        Self { base }
    }

    mmio_rw_base!(0x000 => cs<u32>); // control/status
    mmio_rw_base!(0x004 => clo<u32>); // lower 32bits
    mmio_rw_base!(0x008 => chi<u32>); // higher 32bits
    mmio_rw_base!(0x00c => c0<u32>); // compare 0
    mmio_rw_base!(0x010 => c1<u32>); // compare 1
    mmio_rw_base!(0x014 => c2<u32>); // compare 2
    mmio_rw_base!(0x018 => c3<u32>); // compare 3
}

impl delays::Delays for Delays {
    /// Get System Timer's counter
    fn get_timer_value(&self) -> usize {
        let mut hi: u32 = self.chi().read();
        let mut lo: u32 = self.clo().read();

        if hi != self.chi().read() {
            hi = self.chi().read();
            lo = self.clo().read();
        }

        ((hi as u64) << 32 | lo as u64) as usize
    }

    /// Wait N microsec (with BCM System Timer)
    fn wait_microsec(&self, n: usize) {
        let t = self.get_timer_value();
        // we must check if it's non-zero, because qemu does not emulate
        // system timer, and returning constant zero would mean infinite loop
        if t > 0 {
            while self.get_timer_value() < t + n {}
        }
    }

    fn init(&self) {}
}
