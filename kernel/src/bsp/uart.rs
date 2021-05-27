use crate::{cpuint, driver::uart::UART, global::GlobalVar};
//use alloc::vec::Vec;
use synctools::mcs::{MCSLock, MCSNode};

const UART_CLOCK: usize = 48000000;
const UART_BAUD: usize = 115200;

#[cfg(feature = "pine64")]
type DevUART = crate::driver::uart::sunxi_uart::SunxiUART;

#[cfg(any(feature = "raspi3", feature = "raspi4"))]
type DevUART = crate::driver::uart::pl011::PL011;

impl DevUART where DevUART: UART {}

static UART0: MCSLock<GlobalVar<DevUART>> = MCSLock::new(GlobalVar::UnInit);

pub(super) fn init(uart0: DevUART) {
    let _mask = cpuint::mask();
    let mut node = MCSNode::new();
    let mut lock = UART0.lock(&mut node);
    if let GlobalVar::UnInit = *lock {
        *lock = GlobalVar::Having(uart0);
    } else {
        panic!("initialized twice");
    }
}

pub fn enable_recv_interrupt() {
    let _mask = cpuint::mask();
    let mut node = MCSNode::new();
    let lock = UART0.lock(&mut node);
    if let GlobalVar::Having(uart0) = &*lock {
        uart0.enable_recv_interrupt();
    }
}

pub fn disable_recv_interrupt() {
    let _mask = cpuint::mask();
    let mut node = MCSNode::new();
    let lock = UART0.lock(&mut node);
    if let GlobalVar::Having(uart0) = &*lock {
        uart0.disable_recv_interrupt();
    }
}

/// print characters to serial console
pub fn puts(s: &str) {
    let _mask = cpuint::mask();
    let mut node = MCSNode::new();
    let lock = UART0.lock(&mut node);
    if let GlobalVar::Having(uart0) = &*lock {
        for c in s.bytes() {
            uart0.send(c as u32);
            if c == b'\n' {
                uart0.send(b'\r' as u32);
            }
        }
    }
}

/// print a 64-bit value in hexadecimal to serial console
pub fn hex(h: u64) {
    let _mask = cpuint::mask();
    let mut node = MCSNode::new();
    let lock = UART0.lock(&mut node);
    if let GlobalVar::Having(uart0) = &*lock {
        for i in (0..61).step_by(4).rev() {
            let mut n = (h >> i) & 0xF;
            n += if n > 9 { 0x37 } else { 0x30 };
            uart0.send(n as u32);
        }
    }
}

/// print a 32-bit value in hexadecimal to serial console
pub fn hex32(h: u32) {
    let _mask = cpuint::mask();
    let mut node = MCSNode::new();
    let lock = UART0.lock(&mut node);
    if let GlobalVar::Having(uart0) = &*lock {
        for i in (0..29).step_by(4).rev() {
            let mut n = (h >> i) & 0xF;
            n += if n > 9 { 0x37 } else { 0x30 };
            uart0.send(n as u32);
        }
    }
}

/// print a 8-bit value in binary to serial console
pub fn bin8(b: u8) {
    let _mask = cpuint::mask();
    let mut node = MCSNode::new();
    let lock = UART0.lock(&mut node);
    if let GlobalVar::Having(uart0) = &*lock {
        for i in (0..8).rev() {
            if (1 << i) & b == 0 {
                uart0.send(0x30);
            } else {
                uart0.send(0x31);
            }
        }
    }
}

/// print a 64-bit value in decimal to serial console
pub fn decimal(mut h: u64) {
    let mut num = [0; 32];

    let _mask = cpuint::mask();
    let mut node = MCSNode::new();
    let lock = UART0.lock(&mut node);
    if let GlobalVar::Having(uart0) = &*lock {
        if h == 0 {
            uart0.send('0' as u32);
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
            uart0.send(num[i - 1] as u32);
            i -= 1;
        }
    }
}

pub fn read(v: &mut [u8], echo: fn(&DevUART, u8)) -> usize {
    let mut n = 0;

    let _mask = cpuint::mask();
    let mut node = MCSNode::new();
    let lock = UART0.lock(&mut node);
    if let GlobalVar::Having(uart0) = &*lock {
        loop {
            let c = uart0.recv() as u8;
            v[n] = c;
            n += 1;

            echo(uart0, c);

            if n >= v.len() {
                break;
            }
        }
    }

    n
}

/*
pub fn read_line() -> Vec<u8> {
    let mut res = Vec::new();

    loop {
        let c = recv() as u8;
        if c == b'\r' || c == b'\n' {
            break;
        } else if c == 0x08 || c == 0x7F {
            if !res.is_empty() {
                send(0x08);
                send(b' ' as u32);
                send(0x08);
                res.pop();
            }
        } else if c == b'\t' {
            let c = b' ';
            for _ in 0..8 {
                send(c as u32);
                res.push(c);
            }
        } else if c == 0x15 {
            while !res.is_empty() {
                send(0x08);
                send(b' ' as u32);
                send(0x08);
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
*/
