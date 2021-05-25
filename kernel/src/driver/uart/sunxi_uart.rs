use crate::driver::uart::UART;

pub struct SunxiUART {
    base: usize,
}

impl UART for SunxiUART {
    fn new(base: usize) -> Self {
        Self { base }
    }
    fn send(&self, _c: u32) {}
    fn recv(&self) -> u32 {
        0
    }
    fn enable_recv_interrupt(&self) {}
    fn disable_recv_interrupt(&self) {}
    fn on(&self) {}
    fn off(&self) {}
    fn init(&self, _clock: usize, _baudrate: usize) {}
}
