use core::intrinsics::volatile_load;
use core::intrinsics::volatile_store;

pub fn bit_clear32(ptr: *mut u32, n: u32) {
    unsafe {
        let v = volatile_load(ptr);
        volatile_store(ptr, v & !(1 << n));
    }
}

pub fn bit_set32(ptr: *mut u32, n: u32) {
    unsafe {
        let v = volatile_load(ptr);
        volatile_store(ptr, v | (1 << n));
    }
}
