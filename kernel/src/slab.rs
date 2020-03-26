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
    lock: u64,
    pages: pager::PageManager,
}

unsafe impl GlobalAlloc for Allocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        driver::uart::puts("alloc:\n");
        driver::uart::puts("  layout.size: ");
        driver::uart::decimal(layout.size() as u64);
        driver::uart::puts("\n  layout.align: ");
        driver::uart::decimal(layout.align() as u64);
        driver::uart::puts("\n");

        let _lock = lock::SpinLock::new(&mut SLAB_ALLOC.lock);

        null_mut()
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {}
}

#[global_allocator]
static GLOBAL: Allocator = Allocator;

static mut SLAB_ALLOC: SlabAllocator = SlabAllocator {
    lock: 0,
    pages: pager::PageManager {
        start: 0,
        end: 0,
        vacancy_books: 0,
        vacancy_pages: [0; 64],
        book: [pager::Book{pages: [0; 64]}; 64],
    },
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
}

macro_rules! SlabSmall {
    ($id:ident, $n:expr, $shift:expr) => {
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
        }
    }
}

// l1_bitmap = 0 (initial value)
// l2_bitmap[63] = 0xFFFF FFFF | 0b11 << 32 (initial value)
// size = 16
SlabSmall!(Slab16, 64, 4);

// l1_bitmap = 0xFFFF FFFF (initial value)
// l2_bitmap[31] = 0b111111111 (initial value)
// size = 32
SlabSmall!(Slab32, 32, 5);

// l1_bitmap = 0xFFFF FFFF FFFF (initial value)
// l2_bitmap[15] = 0b111 (initial value)
// size = 64
SlabSmall!(Slab64, 16, 6);

// l1_bitmap = 0xFFFF FFFF FFFF FF (initial value)
// l2_bitmap[7] = 0b1 (initial value)
// size = 128
SlabSmall!(Slab128, 8, 7);

// l1_bitmap = 0xFFFF FFFF FFFF FFF (initial value)
// l2_bitmap[3] = 0b1 (initial value)
// size = 256
SlabSmall!(Slab256, 4, 8);

// l1_bitmap = 0x3FFF FFFF FFFF FFFF (initial value)
// l2_bitmap[1] = 0b1 (initial value)
// size = 512
SlabSmall!(Slab512, 2, 9);

// l1_bitmap = 0x7FFF FFFF FFFF FFFF (initial value)
// l2_bitmap[0] = 0b1 (initial value)
// size = 1024
SlabSmall!(Slab1024, 1, 10);

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
            // +-------------------+
            // |  pointer to slab  |
            // |     (8 bytes)     |
            // +-------------------+
            // |       index       |
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
                    (*mem).slab = self as *mut $id as usize;
                    (*mem).idx1 = idx1;
                }

                &mut (self.buf[idx + 16]) as *mut u8
            }

            /// deallocate the memory region pointed by ptr which is returned by alloc
            fn free(&mut self, ptr: *mut u8) {
                let addr = ptr as usize;
                let idx1 = unsafe { *((addr - 8) as *mut usize) };

                self.l1_bitmap &= !(1 << (63 - idx1))
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
}