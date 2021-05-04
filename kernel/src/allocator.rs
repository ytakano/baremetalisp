use crate::{aarch64::mmu, process::PROCESS_MAX};
use arr_macro::arr;
use core::ptr::null_mut;
use memalloc::Allocator;

const STACK_SIZE: usize = 1024 * 1024 * 2; // 2MiB
const SLAB_SIZE: usize = 1024 * 1024 * 30; // 30MiB
const BUDDY_SIZE: usize = 1024 * 1024 * 32; // 32MiB
const USER_MEM_OFFSET: usize = 1024 * 1024 * 64; // 1TiB
const USER_MEM_SIZE: usize = BUDDY_SIZE + SLAB_SIZE + STACK_SIZE; // must be 64MiB
const KERN_HEAP_OFFSET: usize = 1024 * 1024 * 64; // 64MiB

//#[global_allocator]
//static mut GLOBAL: Allocator = Allocator::new();

static mut ALLOCATORS: [*mut Allocator; PROCESS_MAX] = arr!(null_mut(); 256);

//#[global_allocator]
static mut KENR_ALLOCATOR: Allocator = Allocator::new();

fn kern_offset() -> usize {
    (mmu::get_ram_start() + KERN_HEAP_OFFSET as u64 + mmu::EL1_ADDR_OFFSET) as usize
}

/// Check addr is a kernel's heap address
pub fn is_kern_mem(addr: usize) -> bool {
    let offset = kern_offset();
    offset <= addr && addr < offset + SLAB_SIZE + BUDDY_SIZE
}

pub fn init_kern() {
    let offset = kern_offset();
    unsafe {
        KENR_ALLOCATOR.init_slab(offset, SLAB_SIZE);
        KENR_ALLOCATOR.init_buddy(offset + SLAB_SIZE);
    }
}

fn user_offset(id: u8) -> usize {
    USER_MEM_OFFSET + id as usize * BUDDY_SIZE + SLAB_SIZE + STACK_SIZE
}

/// Check addr is the canary region of id's process
/// If true, stack overflow
pub fn is_user_canary(id: u8, addr: usize) -> bool {
    let offset = user_offset(id);
    addr & !(mmu::PAGESIZE - 1) as usize == offset
}

/// Check addr is a heap address of id's process
pub fn is_user_mem(id: u8, addr: usize) -> bool {
    let offset = user_offset(id);
    offset <= addr && addr < offset + STACK_SIZE + SLAB_SIZE + BUDDY_SIZE
}

/// Memory Layout
/// +-----------------------------+ 1TiB (id = 0)
/// | 2MiB stack space            |
/// +-----------------------------+
/// | 30MiB slab allocator space  |
/// +-----------------------------+
/// | 32MiB buddy allocator space |
/// +-----------------------------+ 1TiB + 64MiB (id = 1)
/// | 2MiB stack space            |
/// +-----------------------------+
/// | 30MiB slab allocator space  |
/// +-----------------------------+
/// | 32MiB buddy allocator space |
/// +-----------------------------+
/// ...
pub fn set_user_allocator(id: u8, ptr: *mut Allocator) {
    unsafe {
        let allc = &mut *ptr;
        let offset = user_offset(id);
        allc.init_slab(offset + STACK_SIZE, SLAB_SIZE);
        allc.init_buddy(offset + STACK_SIZE + SLAB_SIZE);
        ALLOCATORS[id as usize] = ptr;
    }
}
