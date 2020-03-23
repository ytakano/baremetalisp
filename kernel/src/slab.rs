use crate::aarch64::helper::clz;

trait Page {
    fn alloc(&mut self) -> *mut u8;
}

struct Page8 {
    buf: [u64; 8063],
    next: *mut Page8,
    l1_bitmap: [u64; 2],   // l1_bitmap[1] = 0b11 (initial value)
    l2_bitmap: [u64; 126], // l2_bitmap[125] = 1 (initial value)
}

impl Page for Page8 {
    fn alloc(&mut self) -> *mut u8 {
        let idx1 = if self.l1_bitmap[0] == !0 {
            clz(!self.l1_bitmap[1]) + 64
        } else {
            clz(!self.l1_bitmap[0])
        } as usize;

        let idx2 = clz(!self.l2_bitmap[idx1]) as usize;

        let addr = &mut (self.buf[idx1 * 64 + idx2]) as *mut u64 as *mut u8;

        self.l2_bitmap[idx1] |= 1 << (63 - idx2);
        if self.l2_bitmap[idx1] == !0 {
            if idx1 >= 64 {
                self.l1_bitmap[1] |= 1 << (63 - idx1);
            } else {
                self.l1_bitmap[0] |= 1 << (63 - idx1);
            }
        }

        addr
    }
}

struct Page16 {
    buf: [u8; 65008],
    next: *mut Page16,
    l1_bitmap: u64,       // l1_bitmap = 0 (initial value)
    l2_bitmap: [u64; 64], // l2_bitmap[63] = 0xFFFFFFFF | 1 << 32 (initial value)
}

impl Page for Page16 {
    fn alloc(&mut self) -> *mut u8 {
        let idx1 = clz(!self.l1_bitmap) as usize;
        let idx2 = clz(!self.l2_bitmap[idx1]) as usize;

        let addr = &mut (self.buf[idx1 * 16 * 64 + idx2 * 16]) as *mut u8;

        self.l2_bitmap[idx1] |= 1 << (63 - idx2);
        if self.l2_bitmap[idx1] == !0 {
            self.l1_bitmap |= 1 << (63 - idx1);
        }

        addr
    }
}

struct Page32 {
    buf: [u8; 65268],
    next: *mut Page32,
    l1_bitmap: u64,       // l1_bitmap = 0xFFFF FFFF (initial value)
    l2_bitmap: [u64; 32], // l2_bitmap[31] = 0b111111111 (initial value)
}

struct Page64 {
    buf: [u8; 65392],
    next: *mut Page64,
    l1_bitmap: u64,       // l1_bitmap = 0xFFFF FFFF FFFF (initial value)
    l2_bitmap: [u64; 16], // l2_bitmap[15] = 0b111 (initial value)
}

struct Page128 {
    buf: [u8; 65456],
    next: *mut Page128,
    l1_bitmap: u64,      // l1_bitmap = 0xFFFF FFFF FFFF FF (initial value)
    l2_bitmap: [u64; 8], // l2_bitmap[7] = 0b11 (initial value)
}

struct Page256 {
    buf: [u8; 65488],
    next: *mut Page256,
    l1_bitmap: u64,      // l1_bitmap = 0xFFFF FFFF FFFF FFF (initial value)
    l2_bitmap: [u64; 4], // l2_bitmap[3] = 0b1 (initial value)
}

struct Page511 {
    buf: [u8; 65512],
    next: *mut Page511,
    l1_bitmap: [u64; 2], // 0 (initial value)
}

struct Page1023 {
    buf: [u8; 65520],
    next: *mut Page1023,
    l1_bitmap: u64, // l1_bitmap = 0 (initial value)
}

struct Page2047 {
    buf: [u8; 65520],
    next: *mut Page2047,
    l1_bitmap: u64, // l1_bitmap = 0xFFFF FFFF (initial value)
}

struct Page4095 {
    buf: [u8; 65520],
    next: *mut Page4095,
    l1_bitmap: u64, // l1_bitmap = 0xFFFF FFFF FFFF (initial value)
}

struct Page8190 {
    buf: [u8; 65520],
    next: *mut Page8190,
    l1_bitmap: u64, // l1_bitmap = 0xFFFF FFFF FFFF FF (initial value)
}

struct Page32760 {
    buf: [u8; 65520],
    next: *mut Page32760,
    l1_bitmap: u64, // l1_bitmap = 0xFFFF FFFF FFFF FFFC (initial value)
}