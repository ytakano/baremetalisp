use crate::aarch64::lock;
use crate::aarch64::mmu::PAGESIZE;
use crate::driver::{delays, uart};

use alloc::alloc::handle_alloc_error;
use core::alloc::{GlobalAlloc, Layout};

mod buddy;
mod slab;

static mut LOCK_VAR: lock::LockVar = lock::LockVar::new();
static mut BUDDY_ALLOC: buddy::BuddyAlloc = buddy::BuddyAlloc::new(0, 0);

struct Allocator {}

#[global_allocator]
static GLOBAL: Allocator = Allocator {};

unsafe impl GlobalAlloc for Allocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        LOCK_VAR.lock();
        if slab::MAX_SLAB_SIZE >= layout.size() {
            slab::slab_alloc(layout)
        } else {
            match BUDDY_ALLOC.mem_alloc(layout.size()) {
                Some(addr) => addr,
                None => {
                    handle_alloc_error(layout);
                }
            }
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        LOCK_VAR.lock();
        if slab::MAX_SLAB_SIZE >= layout.size() {
            slab::slab_dealloc(ptr, layout)
        } else {
            BUDDY_ALLOC.mem_free(ptr);
        }
    }
}

#[alloc_error_handler]
fn on_oom(layout: Layout) -> ! {
    unsafe { LOCK_VAR.force_unlock() };
    let size = layout.size() as u64;
    uart::puts("memory allocation error: size = ");
    uart::decimal(size);
    uart::puts("\n");
    delays::forever()
}

pub fn init(slab_start: usize, slab_size: usize, buddy_start: usize) {
    unsafe {
        slab::init(slab_start, slab_size);
        BUDDY_ALLOC = buddy::BuddyAlloc::new(PAGESIZE as usize, buddy_start);
    }
}

pub fn test() {
    let mut allc = buddy::BuddyAlloc::new(PAGESIZE as usize, 0);

    let addr1 = match allc.mem_alloc(PAGESIZE as usize * 2) {
        Some(addr) => addr,
        None => {
            uart::puts("failed alloc addr1\n");
            return;
        }
    };

    let addr2 = match allc.mem_alloc(PAGESIZE as usize * 3) {
        Some(addr) => addr,
        None => {
            uart::puts("failed alloc addr2\n");
            return;
        }
    };

    let addr3 = match allc.mem_alloc(PAGESIZE as usize * 8) {
        Some(addr) => addr,
        None => {
            uart::puts("failed alloc addr3\n");
            return;
        }
    };

    allc.print();
    uart::puts("\n");

    allc.mem_free(addr2);
    allc.print();
    uart::puts("\n");

    allc.mem_free(addr1);
    allc.print();
    uart::puts("\n");

    allc.mem_free(addr3);
    allc.print();
}
