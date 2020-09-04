use super::cpu;
use super::mmu::PAGESIZE;

/// clean cache.
/// dc cvac
pub fn clean<T>(obj: &T, size: usize) {
    let addr = obj as *const T as usize;
    let mut base = addr & !(PAGESIZE as usize - 1);

    while base < addr + size {
        base += PAGESIZE as usize;
        unsafe {
            asm!("dc cvac, {}", in(reg) base);
        }
    }

    cpu::dmb_sy();
}

/// flush cache
/// dc civac
pub fn clean_invalidate<T>(obj: &T, size: usize) {
    let addr = obj as *const T as usize;
    let mut base = addr & !(PAGESIZE as usize - 1);

    while base < addr + size {
        base += PAGESIZE as usize;
        unsafe {
            asm!("dc civac, {}", in(reg) base);
        }
    }

    cpu::dmb_sy();
}

/// invalidate cache
/// dc ivac
pub fn invalidate<T>(obj: &T, size: usize) {
    let addr = obj as *const T as usize;
    let mut base = addr & !(PAGESIZE as usize - 1);

    while base < addr + size {
        base += PAGESIZE as usize;
        unsafe {
            asm!("dc ivac, {}", in(reg) base);
        }
    }

    cpu::dmb_sy();
}
