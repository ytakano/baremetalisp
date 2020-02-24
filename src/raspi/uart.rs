extern {
    fn uart_send(s: u32);
}

pub fn puts(s: &str) {
    for c in s.bytes() {
        unsafe { uart_send(c as u32) };
        if c == '\n' as u8 {
            unsafe { uart_send('\r' as u32) };
        }
    }
}

pub fn hex(h : u64) {
    for i in (0..61).step_by(4).rev() {
        let mut n = (h >> i) & 0xF;
        n += if n > 9 { 0x37 } else { 0x30 };
        unsafe { uart_send(n as u32) };
    }
}