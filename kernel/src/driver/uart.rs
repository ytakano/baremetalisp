use super::device::bcm2711;

const UART_CLOCK: u64 = 48000000;

fn send(c : u32) {
    bcm2711::uart::send(c);
}

pub fn init(baudrate: u64) {
    bcm2711::uart::init(UART_CLOCK, baudrate);
}

/// print characters to serial console
pub fn puts(s : &str) {
    for c in s.bytes() {
        send(c as u32);
        if c == '\n' as u8 {
            send('\r' as u32);
        }
    }
}

/// print a 64-bit value in hexadecimal to serial console
pub fn hex(h : u64) {
    for i in (0..61).step_by(4).rev() {
        let mut n = (h >> i) & 0xF;
        n += if n > 9 { 0x37 } else { 0x30 };
        send(n as u32);
    }
}

/// print a 32-bit value in hexadecimal to serial console
pub fn hex32(h : u32) {
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