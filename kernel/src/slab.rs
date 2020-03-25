use crate::aarch64::bits::clz;

trait Slab {
    fn alloc(&mut self) -> *mut u8;
}

macro_rules! SlabSmall {
    ($id:ident, $n:expr) => {
        #[repr(C)]
        struct $id {
            buf: [u8; 65536 - 32 - 8 * $n],
            l1_bitmap: u64,
            l2_bitmap: [u64; $n],
            prev: *mut $id,
            next: *mut $id,
            size: usize
        }

        impl Slab for $id {
            // +-----------------+
            // | pointer to slab |
            // |    (64 bits)    |
            // +-----------------+ <- return value
            // |      data       |
            // |                 |
            fn alloc(&mut self) -> *mut u8 {
                let idx1 = clz(!self.l1_bitmap) as usize;
                let idx2 = clz(!self.l2_bitmap[idx1]) as usize;

                self.l2_bitmap[idx1] |= 1 << (63 - idx2);
                if self.l2_bitmap[idx1] == !0 {
                    self.l1_bitmap |= 1 << (63 - idx1);
                }

                let idx = idx1 * self.size * 64 + idx2 * self.size;
                let ptr = &mut (self.buf[idx]) as *mut u8;
                let ptr64 = ptr as *mut usize;

                // first 64 bits points the slab
                unsafe { *ptr64 = self as *mut $id as usize; }

                &mut (self.buf[idx + 8]) as *mut u8
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

#[repr(C)]
struct SlabMemory {
    slab: usize,
    idx1: usize,
}

macro_rules! SlabLarge {
    ($id:ident) => {
        #[repr(C)]
        struct $id {
            buf: [u8; 65504],
            prev: *mut $id,
            next: *mut $id,
            l1_bitmap: u64,
            size: usize,
        }

        impl Slab for $id {
            // +-----------------+
            // | pointer to slab |
            // |    (64 bits)    |
            // +-----------------+
            // |     index       |
            // |    (64 bits)    |
            // +-----------------+ <- return value
            // |      data       |
            // |                 |
            fn alloc(&mut self) -> *mut u8 {
                let idx1 = clz(!self.l1_bitmap) as usize;
                self.l1_bitmap |= 1 << (63 - idx1);

                let idx = idx1 * self.size * 64;
                let ptr = &mut (self.buf[idx]) as *mut u8;
                let mem = ptr as *mut SlabMemory;

                // first 128 bits contain meta information
                unsafe {
                    (*mem).slab = self as *mut $id as usize;
                    (*mem).idx1 = idx1;
                }

                &mut (self.buf[idx + 16]) as *mut u8
            }
        }
    }
}

// l1_bitmap = 0xFFFF FFFF (initial value)
// size = 2047
SlabLarge!(Slab2047);

// l1_bitmap = 0xFFFF FFFF FFFF (initial value)
// size = 4094
SlabLarge!(Slab4094);

// l1_bitmap = 0xFFFF FFFF FFFF FF (initial value)
// size = 8188
SlabLarge!(Slab8188);

// l1_bitmap = 0xFFFF FFFF FFFF FFF (initial value)
// size = 16376
SlabLarge!(Slab16376);

// l1_bitmap = 0x3FFF FFFF FFFF FFFF (initial value)
// size = 32752
SlabLarge!(Slab32752);

#[repr(C)]
struct Slab65512 {
    buf: [u8; 65512],
    prev: *mut Slab65512,
    next: *mut Slab65512,
    size: usize,
}

impl Slab for Slab65512 {
    // +-----------------+
    // | pointer to slab |
    // |    (64 bits)    |
    // +-----------------+ <- return value
    // |      data       |
    // |                 |
    fn alloc(&mut self) -> *mut u8 {
        let ptr = &mut (self.buf[0]) as *mut u8;
        let ptr64 = ptr as *mut usize;

        // first 64 bits points the slab
        unsafe { *ptr64 = self as *mut Slab65512 as usize; }

        &mut (self.buf[8]) as *mut u8
    }
}