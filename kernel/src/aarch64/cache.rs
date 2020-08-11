use super::cpu;
use super::mmu::PAGESIZE;

pub fn clean(addr: usize, size: usize) {
    let mut base = addr & !(PAGESIZE as usize - 1);

    while base < addr + size {
        base += PAGESIZE as usize;
        unsafe {
            asm!("dc cmvac, {}", in(reg) base);
        }
    }

    cpu::dmb_sy();
}

pub fn clean_invalidate(addr: usize, size: usize) {
    let mut base = addr & !(PAGESIZE as usize - 1);

    while base < addr + size {
        base += PAGESIZE as usize;
        unsafe {
            asm!("dc cimvac, {}", in(reg) base);
        }
    }

    cpu::dmb_sy();
}

pub fn invalidate(addr: usize, size: usize) {
    let mut base = addr & !(PAGESIZE as usize - 1);

    while base < addr + size {
        base += PAGESIZE as usize;
        unsafe {
            asm!("dc imvac, {}", in(reg) base);
        }
    }

    cpu::dmb_sy();
}
