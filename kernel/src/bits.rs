use core::ptr::{read_volatile, write_volatile};

pub fn bit_clear32(ptr: *mut u32, n: u32) {
    unsafe {
        let v = read_volatile(ptr);
        write_volatile(ptr, v & !(1 << n));
    }
}

pub fn bit_set32(ptr: *mut u32, n: u32) {
    unsafe {
        let v = read_volatile(ptr);
        write_volatile(ptr, v | (1 << n));
    }
}

pub fn clrbits_32(ptr: *mut u32, clear: u32) {
    unsafe {
        let v = read_volatile(ptr);
        write_volatile(ptr, v & !clear);
    }
}

pub fn setbits_32(ptr: *mut u32, set: u32) {
    unsafe {
        let v = read_volatile(ptr);
        write_volatile(ptr, v | set);
    }
}

pub fn clrsetbits_32(ptr: *mut u32, clear: u32, set: u32) {
    unsafe {
        let v = read_volatile(ptr);
        write_volatile(ptr, (v & !clear) | set);
    }
}

pub fn genmask32(h: u32, l: u32) -> u32 {
    ((0xFFFFFFFF) << (l)) & (0xFFFFFFFF >> (32 - 1 - (h)))
}
