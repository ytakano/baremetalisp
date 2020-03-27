use core::alloc::{GlobalAlloc, Layout};
use core::ptr::null_mut;

use crate::driver;
use crate::aarch64::bits::clz;
use crate::pager;
use crate::aarch64::lock;

extern "C" {
    static mut __el0_heap_start: u64;
    static mut __el0_heap_end: u64;
}

struct Allocator;

struct SlabAllocator {
    lock: lock::LockVar,
    pages: pager::PageManager,

       slab16_partial: *mut    Slab16,
       slab32_partial: *mut    Slab32,
       slab64_partial: *mut    Slab64,
      slab128_partial: *mut   Slab128,
      slab256_partial: *mut   Slab256,
      slab512_partial: *mut   Slab512,
     slab1024_partial: *mut  Slab1024,
     slab2040_partial: *mut  Slab2040,
     slab4088_partial: *mut  Slab4088,
     slab8184_partial: *mut  Slab8184,
    slab16376_partial: *mut Slab16376,
    slab32752_partial: *mut Slab32752,

       slab16_full: *mut    Slab16,
       slab32_full: *mut    Slab32,
       slab64_full: *mut    Slab64,
      slab128_full: *mut   Slab128,
      slab256_full: *mut   Slab256,
      slab512_full: *mut   Slab512,
     slab1024_full: *mut  Slab1024,
     slab2040_full: *mut  Slab2040,
     slab4088_full: *mut  Slab4088,
     slab8184_full: *mut  Slab8184,
    slab16376_full: *mut Slab16376,
    slab32752_full: *mut Slab32752,
    slab65512_full: *mut Slab65512,
}

macro_rules! AllocMemory {
    ($t:ident, $slab_partial:ident, $slab_full:ident) => {
        let _lock = SLAB_ALLOC.lock.lock();

        match SLAB_ALLOC.$slab_partial.as_mut() {
            Some(partial) => {
                let ret = partial.alloc();
                if partial.is_full() {
                    let ptr = SLAB_ALLOC.$slab_partial;
                    match partial.next.as_mut() {
                        Some(next) => {
                            next.prev = null_mut();
                        }
                        None => {}
                    }

                    SLAB_ALLOC.$slab_partial = partial.next;
                    match SLAB_ALLOC.$slab_full.as_mut() {
                        Some(full) => {
                            full.prev = ptr;
                        }
                        None => {}
                    }

                    partial.next = SLAB_ALLOC.$slab_full;
                    SLAB_ALLOC.$slab_full = ptr;
                }
                return ret;
            }
            None => {
                match SLAB_ALLOC.pages.alloc() {
                    Some(addr) => {
                        let ptr = addr as *mut $t;
                        match ptr.as_mut() {
                            Some(slab) => {
                                slab.init();
                                let ret = slab.alloc();
                                if slab.is_full() {
                                    // for only Slab65512
                                    match SLAB_ALLOC.$slab_full.as_mut() {
                                        Some(full) => {
                                            full.prev = ptr;
                                        }
                                        None => {}
                                    }
                                    slab.next = SLAB_ALLOC.$slab_full;
                                    SLAB_ALLOC.$slab_full = ptr;
                                } else {
                                    SLAB_ALLOC.$slab_partial = ptr;
                                }
                                return ret;
                            }
                            None => {
                                return null_mut();
                            }
                        }
                    }
                    None => {
                        return null_mut();
                    }
                }
            }
        }
    }
}

unsafe impl GlobalAlloc for Allocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        driver::uart::puts("alloc:\n");
        driver::uart::puts("  layout.size: ");
        driver::uart::decimal(layout.size() as u64);
        driver::uart::puts("\n  layout.align: ");
        driver::uart::decimal(layout.align() as u64);
        driver::uart::puts("\n");

        let size = layout.size();
        let n = clz(size as u64 + 8 - 1);

        if n < 60 {
            // Slab16
        } else {
            match n {
                61 => {
                    AllocMemory!(Slab16, slab16_partial, slab16_full);
                }
                60 => {
                    AllocMemory!(Slab16, slab16_partial, slab16_full);
                }
                59 => {
                    AllocMemory!(Slab32, slab32_partial, slab32_full);
                }
                58 => {
                    AllocMemory!(Slab64, slab64_partial, slab64_full);
                }
                57 => {
                    AllocMemory!(Slab128, slab128_partial, slab128_full);
                }
                56 => {
                    AllocMemory!(Slab256, slab256_partial, slab256_full);
                }
                55 => {
                    AllocMemory!(Slab1024, slab1024_partial, slab1024_full);
                }
                _ => {
                    if size <= 4088 - 16 {
                        if size <= 2040 - 16 {
                            // Slab2040
                        } else {
                            // Slab4088
                        }
                    } else {
                        if size <= 16376 - 16 {
                            if size <= 8184 - 16 {
                                // Slab8184
                            } else {
                                // Slab16376
                            }
                        } else {
                            if size <= 32752 - 16 {
                                // Slab32752
                            } else if size <= 65512 - 16 {
                                // Slab65512
                            } else {
                                return null_mut();
                            }
                        }
                    }
                }
            }
        }

        null_mut()
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {}
}

#[global_allocator]
static GLOBAL: Allocator = Allocator;

static mut SLAB_ALLOC: SlabAllocator = SlabAllocator {
    lock: lock::LockVar::new(),
    pages: pager::PageManager::new(),
       slab16_partial: null_mut(),
       slab32_partial: null_mut(),
       slab64_partial: null_mut(),
      slab128_partial: null_mut(),
      slab256_partial: null_mut(),
      slab512_partial: null_mut(),
     slab1024_partial: null_mut(),
     slab2040_partial: null_mut(),
     slab4088_partial: null_mut(),
     slab8184_partial: null_mut(),
    slab16376_partial: null_mut(),
    slab32752_partial: null_mut(),
       slab16_full: null_mut(),
       slab32_full: null_mut(),
       slab64_full: null_mut(),
      slab128_full: null_mut(),
      slab256_full: null_mut(),
      slab512_full: null_mut(),
     slab1024_full: null_mut(),
     slab2040_full: null_mut(),
     slab4088_full: null_mut(),
     slab8184_full: null_mut(),
    slab16376_full: null_mut(),
    slab32752_full: null_mut(),
    slab65512_full: null_mut(),
};

#[alloc_error_handler]
fn on_oom(_layout: Layout) -> ! {
    loop {}
}

pub fn init() {
    unsafe {
        SLAB_ALLOC.pages.start = __el0_heap_start as usize;
        SLAB_ALLOC.pages.end   = __el0_heap_end as usize;
    }
}

trait Slab {
    fn alloc(&mut self) -> *mut u8;
    fn free(&mut self, ptr: *mut u8);
    fn is_full(&self) -> bool;
    fn init(&mut self);
}

macro_rules! SlabSmall {
    ($id:ident, $n:expr, $shift:expr, $l1val:expr, $l2val:expr, $size:expr) => {
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
            // +------------------+
            // | pointer to slab  |
            // |    (8 bytes)     |
            // +------------------+ <- return value
            // |       data       |
            // | (size - 8 bytes) |
            // |                  |
            /// allocate a memory region whose size is self.size - 8 bytes
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

            /// deallocate the memory region pointed by ptr which is returned by alloc
            fn free(&mut self, ptr: *mut u8) {
                let addr = ptr as usize - 8;
                let org = self as *mut $id as usize;
                let len = addr - org;
                let idx = (len >> $shift) as usize;

                let idx1 = idx >> 6; // divide by 64
                let idx2 = idx & 0b111111;

                self.l1_bitmap &= !(1 << (63 - idx1));
                self.l2_bitmap[idx1] &= !(1 << (63 - idx2));
            }

            fn is_full(&self) -> bool {
                self.l1_bitmap == !0
            }

            fn init(&mut self) {
                self.l1_bitmap = $l1val;
                for it in self.l2_bitmap.iter_mut() {
                    *it = 0;
                }
                self.l2_bitmap[$n - 1] = $l2val;
                self.prev = null_mut();
                self.next = null_mut();
                self.size = $size;
            }
        }
    }
}

// l1_bitmap = 0 (initial value)
// l2_bitmap[63] = 0xFFFF FFFF | 0b11 << 32 (initial value)
// size = 16
SlabSmall!(Slab16, 64, 4, 0, 0xFFFFFFFF, 16);

// l1_bitmap = 0xFFFF FFFF (initial value)
// l2_bitmap[31] = 0b111111111 (initial value)
// size = 32
SlabSmall!(Slab32, 32, 5, 0xFFFFFFFF, 0b111111111, 32);

// l1_bitmap = 0xFFFF FFFF FFFF (initial value)
// l2_bitmap[15] = 0b111 (initial value)
// size = 64
SlabSmall!(Slab64, 16, 6, 0xFFFFFFFFFFFF, 0b111, 64);

// l1_bitmap = 0xFFFF FFFF FFFF FF (initial value)
// l2_bitmap[7] = 0b1 (initial value)
// size = 128
SlabSmall!(Slab128, 8, 7, 0xFFFFFFFFFFFFFF, 1, 128);

// l1_bitmap = 0xFFFF FFFF FFFF FFF (initial value)
// l2_bitmap[3] = 0b1 (initial value)
// size = 256
SlabSmall!(Slab256, 4, 8, 0xFFFFFFFFFFFFFFF, 1, 256);

// l1_bitmap = 0x3FFF FFFF FFFF FFFF (initial value)
// l2_bitmap[1] = 0b1 (initial value)
// size = 512
SlabSmall!(Slab512, 2, 9, 0x3FFFFFFFFFFFFFFF, 1, 512);

// l1_bitmap = 0x7FFF FFFF FFFF FFFF (initial value)
// l2_bitmap[0] = 0b1 (initial value)
// size = 1024
SlabSmall!(Slab1024, 1, 10, 0x7FFFFFFFFFFFFFFF, 1, 1024);

#[repr(C)]
struct SlabMemory {
    idx1: usize,
    slab: usize,
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
            // +-------------------+
            // |       index       |
            // |     (8 bytes)     |
            // +-------------------+
            // |  pointer to slab  |
            // |     (8 bytes)     |
            // +-------------------+ <- return value
            // |       data        |
            // | (size - 16 bytes) |
            // |                   |
            /// allocate a memory region whose size is self.size - 16 bytes
            fn alloc(&mut self) -> *mut u8 {
                let idx1 = clz(!self.l1_bitmap) as usize;
                self.l1_bitmap |= 1 << (63 - idx1);

                let idx = idx1 * self.size * 64;
                let ptr = &mut (self.buf[idx]) as *mut u8;
                let mem = ptr as *mut SlabMemory;

                // first 128 bits contain meta information
                unsafe {
                    (*mem).idx1 = idx1;
                    (*mem).slab = self as *mut $id as usize;
                }

                &mut (self.buf[idx + 16]) as *mut u8
            }

            /// deallocate the memory region pointed by ptr which is returned by alloc
            fn free(&mut self, ptr: *mut u8) {
                let addr = ptr as usize;
                let idx1 = unsafe { *((addr - 16) as *mut usize) };

                self.l1_bitmap &= !(1 << (63 - idx1))
            }

            fn is_full(&self) -> bool {
                self.l1_bitmap == !0
            }

            fn init(&mut self) {
                // TODO
            }
        }
    }
}

// l1_bitmap = 0xFFFF FFFF (initial value)
// size = 2040
SlabLarge!(Slab2040);

// l1_bitmap = 0xFFFF FFFF FFFF (initial value)
// size = 4088
SlabLarge!(Slab4088);

// l1_bitmap = 0xFFFF FFFF FFFF FF (initial value)
// size = 8184
SlabLarge!(Slab8184);

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
    size: usize, // must be 65512
}

impl Slab for Slab65512 {
    // +------------------+
    // | pointer to slab  |
    // |    (8 bytes)     |
    // +------------------+ <- return value
    // |       data       |
    // | (size - 8 bytes) |
    // |                  |
    /// allocate a memory region whose size is 65504 bytes
    fn alloc(&mut self) -> *mut u8 {
        let ptr = &mut (self.buf[0]) as *mut u8;
        let ptr64 = ptr as *mut usize;

        // first 64 bits points the slab
        unsafe { *ptr64 = self as *mut Slab65512 as usize; }

        &mut (self.buf[8]) as *mut u8
    }

    /// do nothing
    fn free(&mut self, _ptr: *mut u8) {
    }

    fn is_full(&self) -> bool {
        true
    }

    fn init(&mut self) {
        // TODO
    }
}