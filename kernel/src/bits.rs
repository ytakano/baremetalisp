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

pub fn genmask32(h: u32, l: u32) -> u32 {
    ((0xFFFFFFFF) << (l)) & (0xFFFFFFFF >> (32 - 1 - (h)))
}
