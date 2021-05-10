use crate::driver::uart;

const KEY_WIDTH: usize = 24;

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
