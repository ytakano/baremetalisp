use core::{
    ops::{BitAnd, BitOr, Not},
    ptr::{read_volatile, write_volatile},
};

pub struct MMIO<T>(*mut T);

impl<T: Not<Output = T> + BitOr<Output = T> + BitAnd<Output = T>> MMIO<T> {
    pub fn new(ptr: *mut T) -> Self {
        MMIO(ptr)
    }

    pub fn write(&self, n: T) {
        unsafe { write_volatile(self.0, n) };
    }

    pub fn read(&self) -> T {
        unsafe { read_volatile(self.0) }
    }

    pub fn setbits(&self, mask: T) {
        let old = self.read();
        self.write(old | mask);
    }

    pub fn clrbits(&self, mask: T) {
        let old = self.read();
        self.write(old & !mask);
    }
}
