use super::cpu;
use super::mmu::PAGESIZE;

/// clean cache.
/// dc cmvac
pub fn clean<T>(obj: &mut T, size: usize) {
    let addr = obj as *mut T as usize;
    let mut base = addr & !(PAGESIZE as usize - 1);

    while base < addr + size {
        base += PAGESIZE as usize;
        unsafe {
            asm!("dc cmvac, {}", in(reg) base);
        }
    }

    cpu::dmb_sy();
}

/// flush cache
/// dc cimvac
pub fn clean_invalidate<T>(obj: &mut T, size: usize) {
    let addr = obj as *mut T as usize;
    let mut base = addr & !(PAGESIZE as usize - 1);

    while base < addr + size {
        base += PAGESIZE as usize;
        unsafe {
            asm!("dc cimvac, {}", in(reg) base);
        }
    }

    cpu::dmb_sy();
}

/// invalidate cache
/// dc imvac
pub fn invalidate<T>(obj: &mut T, size: usize) {
    let addr = obj as *mut T as usize;
    let mut base = addr & !(PAGESIZE as usize - 1);

    while base < addr + size {
        base += PAGESIZE as usize;
        unsafe {
            asm!("dc imvac, {}", in(reg) base);
        }
    }

    cpu::dmb_sy();
}
