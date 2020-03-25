use crate::aarch64::bits::clz;

trait Slab {
    fn alloc(&mut self) -> *mut u8;
}

macro_rules! SlabSmall {
    ($id:ident, $n:expr) => {
        struct $id {
            buf: [u8; 65536 - 32 - 8 * $n],
            l1_bitmap: u64,
            l2_bitmap: [u64; $n],
            prev: *mut $id,
            next: *mut $id,
            size: usize
        }

        impl Slab for $id {
            fn alloc(&mut self) -> *mut u8 {
                let idx1 = clz(!self.l1_bitmap) as usize;
                let idx2 = clz(!self.l2_bitmap[idx1]) as usize;

                self.l2_bitmap[idx1] |= 1 << (63 - idx2);
                if self.l2_bitmap[idx1] == !0 {
                    self.l1_bitmap |= 1 << (63 - idx1);
                }

                &mut (self.buf[idx1 * self.size * 64 + idx2 * self.size]) as *mut u8
            }
        }
    }
}

// l1_bitmap = 0 (initial value)
// l2_bitmap[63] = 0xFFFF FFFF | 0b11 << 32 (initial value)
// size = 16
SlabSmall!(Slab16, 64);

// l1_bitmap = 0xFFFF FFFF (initial value)
// l2_bitmap[31] = 0b111111111 (initial value)
// size = 32
SlabSmall!(Slab32, 32);

// l1_bitmap = 0xFFFF FFFF FFFF (initial value)
// l2_bitmap[15] = 0b111 (initial value)
// size = 64
SlabSmall!(Slab64, 16);

// l1_bitmap = 0xFFFF FFFF FFFF FF (initial value)
// l2_bitmap[7] = 0b1 (initial value)
// size = 128
SlabSmall!(Slab128, 8);

// l1_bitmap = 0xFFFF FFFF FFFF FFF (initial value)
// l2_bitmap[3] = 0b1 (initial value)
// size = 256
SlabSmall!(Slab256, 4);

// l1_bitmap = 0x3FFF FFFF FFFF FFFF (initial value)
// l2_bitmap[1] = 0b1 (initial value)
// size = 512
SlabSmall!(Slab512, 2);

// l1_bitmap = 0x7FFF FFFF FFFF FFFF (initial value)
// l2_bitmap[0] = 0b1 (initial value)
// size = 1024
SlabSmall!(Slab1024, 1);

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