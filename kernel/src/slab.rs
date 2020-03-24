use crate::aarch64::bits::clz;

trait Slab {
    fn alloc(&mut self) -> *mut u8;
}

struct Slab8 {
    buf: [u64; 8063],
    next: *mut Slab8,
    l1_bitmap: [u64; 2],   // l1_bitmap[1] = 0b11 (initial value)
    l2_bitmap: [u64; 126], // l2_bitmap[125] = 1 (initial value)
}

impl Slab for Slab8 {
    fn alloc(&mut self) -> *mut u8 {
        let idx1 = if self.l1_bitmap[0] == !0 {
            clz(!self.l1_bitmap[1]) + 64
        } else {
            clz(!self.l1_bitmap[0])
        } as usize;

        let idx2 = clz(!self.l2_bitmap[idx1]) as usize;

        self.l2_bitmap[idx1] |= 1 << (63 - idx2);
        if self.l2_bitmap[idx1] == !0 {
            if idx1 >= 64 {
                self.l1_bitmap[1] |= 1 << (63 - idx1);
            } else {
                self.l1_bitmap[0] |= 1 << (63 - idx1);
            }
        }

        &mut (self.buf[idx1 * 64 + idx2]) as *mut u64 as *mut u8
    }
}

macro_rules! SlabSmall {
    ($id:ident, $n:expr, $size:expr) => {
        struct $id {
            buf: [u8; 65536 - 16 - 8 * $n],
            next: *mut $id,
            l1_bitmap: u64,
            l2_bitmap: [u64; $n]
        }

        impl Slab for $id {
            fn alloc(&mut self) -> *mut u8 {
                let idx1 = clz(!self.l1_bitmap) as usize;
                let idx2 = clz(!self.l2_bitmap[idx1]) as usize;

                self.l2_bitmap[idx1] |= 1 << (63 - idx2);
                if self.l2_bitmap[idx1] == !0 {
                    self.l1_bitmap |= 1 << (63 - idx1);
                }

                &mut (self.buf[idx1 * $size * 64 + idx2 * $size]) as *mut u8
            }
        }
    }
}

// l1_bitmap = 0 (initial value)
// l2_bitmap[63] = 0xFFFF FFFF | 1 << 32 (initial value)
SlabSmall!(Slab16, 64, 16);

// l1_bitmap = 0xFFFF FFFF (initial value)
// l2_bitmap[31] = 0b111111111 (initial value)
SlabSmall!(Slab32, 32, 32);

// l1_bitmap = 0xFFFF FFFF FFFF (initial value)
// l2_bitmap[15] = 0b111 (initial value)
SlabSmall!(Slab64, 16, 64);

// l1_bitmap = 0xFFFF FFFF FFFF FF (initial value)
// l2_bitmap[7] = 0b11 (initial value)
SlabSmall!(Slab128, 8, 128);

// l1_bitmap = 0xFFFF FFFF FFFF FFF (initial value)
// l2_bitmap[3] = 0b1 (initial value)
SlabSmall!(Slab256, 4, 256);

struct Slab511 {
    buf: [u8; 65512],
    next: *mut Slab511,
    l1_bitmap: [u64; 2], // 0 (initial value)
}

impl Slab for Slab511 {
    fn alloc(&mut self) -> *mut u8 {
        let idx1 = if self.l1_bitmap[0] == !0 {
            clz(!self.l1_bitmap[1]) + 64
        } else {
            clz(!self.l1_bitmap[0])
        } as usize;

        if idx1 < 64 {
            self.l1_bitmap[0] |= 1 << (63 - idx1);
        } else {
            self.l1_bitmap[1] |= 1 << (63 - (idx1 - 64));
        }

         &mut (self.buf[idx1 * 511 * 64]) as *mut u8
    }
}

macro_rules! SlabLarge {
    ($id:ident, $size:expr) => {
        struct $id {
            buf: [u8; 65520],
            next: *mut $id,
            l1_bitmap: u64,
        }

        impl Slab for $id {
            fn alloc(&mut self) -> *mut u8 {
                let idx1 = clz(!self.l1_bitmap) as usize;
                self.l1_bitmap |= 1 << (63 - idx1);
                &mut (self.buf[idx1 * $size * 64]) as *mut u8
            }
        }
    }
}

// l1_bitmap = 0 (initial value)
SlabLarge!(Slab1023, 1023);

// l1_bitmap = 0xFFFF FFFF (initial value)
SlabLarge!(Slab2047, 2047);

// l1_bitmap = 0xFFFF FFFF FFFF (initial value)
SlabLarge!(Slab4095, 4095);

// l1_bitmap = 0xFFFF FFFF FFFF FF (initial value)
SlabLarge!(Slab8190, 8190);

// l1_bitmap = 0xFFFF FFFF FFFF FFF (initial value)
SlabLarge!(Slab16380, 16380);

// l1_bitmap = 0xFFFF FFFF FFFF FFFC (initial value)
SlabLarge!(Slab32760, 32760);

struct Slab65528 {
    buf: [u8; 65528],
    next: *mut Slab65528,
}

impl Slab for Slab65528 {
    fn alloc(&mut self) -> *mut u8 {
        &mut (self.buf[0]) as *mut u8
    }
}