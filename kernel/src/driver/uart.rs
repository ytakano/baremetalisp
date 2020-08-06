#[cfg(any(feature = "raspi3", feature = "raspi4"))]
use super::device::raspi::uart;

#[cfg(feature = "pine64")]
use super::device::allwinner::uart;

use alloc::vec::Vec;

const UART_CLOCK: u64 = 48000000;
const UART_BAUD: u64 = 115200;

fn send(c: u32) {
    uart::send(c);
}

pub fn recv() -> u32 {
    return uart::recv();
}

pub fn init() {
    uart::init(UART_CLOCK, UART_BAUD);
}

/// print characters to serial console
pub fn puts(s: &str) {
    for c in s.bytes() {
        send(c as u32);
        if c == '\n' as u8 {
            send('\r' as u32);
        }
    }
}

/// print a 64-bit value in hexadecimal to serial console
pub fn hex(h: u64) {
    for i in (0..61).step_by(4).rev() {
        let mut n = (h >> i) & 0xF;
        n += if n > 9 { 0x37 } else { 0x30 };
        send(n as u32);
    }
}

/// print a 32-bit value in hexadecimal to serial console
pub fn hex32(h: u32) {
    for i in (0..29).step_by(4).rev() {
        let mut n = (h >> i) & 0xF;
        n += if n > 9 { 0x37 } else { 0x30 };
        send(n as u32);
    }
}

/// print a 64-bit value in decimal to serial console
pub fn decimal(mut h: u64) {
    let mut num = [0; 32];

    if h == 0 {
        send('0' as u32);
        return;
    }

    let mut i = 0;
    while h > 0 {
        let n = h % 10;
        h /= 10;
        num[i] = n + 0x30;
        i += 1;
    }

    while i > 0 {
        send(num[i - 1] as u32);
        i -= 1;
    }
}

pub fn read_line() -> Vec<u8> {
    let mut res = Vec::new();

    loop {
        let c = recv() as u8;
        if c == '\r' as u8 || c == '\n' as u8 {
            break;
        } else if c == 0x08 || c == 0x7F {
            if !res.is_empty() {
                send(0x08 as u32);
                send(' ' as u32);
                send(0x08 as u32);
                res.pop();
            }
        } else if c == '\t' as u8 {
            let c = ' ' as u8;
            for _ in 0..8 {
                send(c as u32);
                res.push(c);
            }
        } else if c == 0x15 {
            while !res.is_empty() {
                send(0x08 as u32);
                send(' ' as u32);
                send(0x08 as u32);
                res.pop();
            }
        } else {
            send(c as u32);
            res.push(c);
        }
    }

    puts("\n");

    res
}
