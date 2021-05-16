use alloc::vec::Vec;
use synctools::mcs;

const UART_CLOCK: usize = 48000000;
const UART_BAUD: usize = 115200;

pub(super) trait UART {
    fn send(c: u32);
    fn recv() -> u32;
    fn init(clock: usize, baudrate: usize);
}

static mut LOCK: mcs::MCSLock<()> = mcs::MCSLock::new(());

#[cfg(feature = "pine64")]
type DevUART = super::device::allwinner::uart::A64UART;

#[cfg(any(feature = "raspi3", feature = "raspi4"))]
type DevUART = super::device::raspi::uart::RaspiUART;

impl DevUART where DevUART: UART {}

fn send(c: u32) {
    DevUART::send(c);
}

fn recv() -> u32 {
    DevUART::recv()
}

pub(crate) fn enable_recv_int() {
    //let mut node = mcs::MCSNode::new();
    //let _lock = lock(&mut node);
    //uart::enable_recv_int();
    todo!("")
}

pub(crate) fn init() {
    DevUART::init(UART_CLOCK, UART_BAUD);
}

fn lock<'_t>(node: &'_ mut mcs::MCSNode<()>) -> Option<mcs::MCSLockGuard<'_, ()>> {
    Some(unsafe { LOCK.lock(node) })
}

/// print characters to serial console
pub(crate) fn puts(s: &str) {
    let mut node = mcs::MCSNode::new();
    let _lock = lock(&mut node);
    for c in s.bytes() {
        send(c as u32);
        if c == b'\n' {
            send(b'\r' as u32);
        }
    }
}

/// print a 64-bit value in hexadecimal to serial console
pub(crate) fn hex(h: u64) {
    let mut node = mcs::MCSNode::new();
    let _lock = lock(&mut node);
    for i in (0..61).step_by(4).rev() {
        let mut n = (h >> i) & 0xF;
        n += if n > 9 { 0x37 } else { 0x30 };
        send(n as u32);
    }
}

/// print a 32-bit value in hexadecimal to serial console
pub(crate) fn hex32(h: u32) {
    let mut node = mcs::MCSNode::new();
    let _lock = lock(&mut node);
    for i in (0..29).step_by(4).rev() {
        let mut n = (h >> i) & 0xF;
        n += if n > 9 { 0x37 } else { 0x30 };
        send(n as u32);
    }
}

/// print a 8-bit value in binary to serial console
pub(crate) fn bin8(b: u8) {
    let mut node = mcs::MCSNode::new();
    let _lock = lock(&mut node);
    for i in (0..8).rev() {
        if (1 << i) & b == 0 {
            send(0x30);
        } else {
            send(0x31);
        }
    }
}

/// print a 64-bit value in decimal to serial console
pub(crate) fn decimal(mut h: u64) {
    let mut node = mcs::MCSNode::new();
    let _lock = lock(&mut node);
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

pub(crate) fn read_line() -> Vec<u8> {
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
