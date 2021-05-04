use crate::driver::uart;

const KEY_WIDTH: usize = 16;

pub fn msg(key: &str, val: &str) {
    uart::puts("[");
    uart::puts(key);
    for _ in key.len()..KEY_WIDTH {
        uart::puts(" ");
    }
    uart::puts("] ");
    uart::puts(val);
    uart::puts("\n");
}

pub fn decimal(key: &str, n: u64) {
    uart::puts("[");
    uart::puts(key);
    for _ in key.len()..KEY_WIDTH {
        uart::puts(" ");
    }
    uart::puts("] ");
    uart::decimal(n);
    uart::puts("\n");
}

pub fn hex32(key: &str, n: u32) {
    uart::puts("[");
    uart::puts(key);
    for _ in key.len()..KEY_WIDTH {
        uart::puts(" ");
    }
    uart::puts("] 0x");
    uart::hex32(n);
    uart::puts("\n");
}

pub fn hex64(key: &str, n: u64) {
    uart::puts("[");
    uart::puts(key);
    for _ in key.len()..KEY_WIDTH {
        uart::puts(" ");
    }
    uart::puts("] 0x");
    uart::hex(n);
    uart::puts("\n");
}

pub fn bin8(key: &str, n: u8) {
    uart::puts("[");
    uart::puts(key);
    for _ in key.len()..KEY_WIDTH {
        uart::puts(" ");
    }
    uart::puts("] 0b");
    uart::bin8(n);
    uart::puts("\n");
}
use core::fmt::{self, Write};

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    ($fmt:expr) => (print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => (print!(concat!($fmt, "\n"), $($arg)*));
}

pub fn _print(args: fmt::Arguments) {
    let mut writer = UartWriter {};
    writer.write_fmt(args).unwrap();
}

struct UartWriter;

impl Write for UartWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        uart::puts(s);
        Ok(())
    }
}
