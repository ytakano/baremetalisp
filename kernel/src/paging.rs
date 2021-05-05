use crate::{
    aarch64::{int::InterMask, mmu},
    allocator,
    global::GlobalVar,
};
use memalloc::pager::PageManager;
use synctools::mcs::{MCSLock, MCSNode};

static PAGER: MCSLock<GlobalVar<PageManager>> = MCSLock::new(GlobalVar::UnInit);

pub fn init(start: usize, end: usize) {
    let mut pager = PageManager::new();
    pager.set_range(start, end);

    let mut node = MCSNode::new();
    let mut lock = PAGER.lock(&mut node);
    if let GlobalVar::UnInit = *lock {
        *lock = GlobalVar::Having(pager);
    } else {
        panic!("initialized twice");
    }
}

pub fn map(vm_addr: usize, is_kern: bool) {
    if is_kern {
        if !allocator::is_kern_mem(vm_addr) {
            panic!("invalid memory region");
        }
    } else {
        todo!("check user memory");
    }

    // disable interrupts
    let _mask = InterMask::new();

    let mut node = MCSNode::new();
    let mut lock = PAGER.lock(&mut node);

    if let GlobalVar::Having(pager) = &mut *lock {
        if let Some(phy_addr) = pager.alloc() {
            lock.unlock();
            if is_kern {
                let mut ttbr1 = mmu::get_ttbr1();
                ttbr1.map(vm_addr as u64, phy_addr as u64, mmu::kernel_page_flag());
                mmu::tlb_flush_addr(vm_addr);
            } else {
                let mut ttbr0 = mmu::get_ttbr0();
                ttbr0.map(vm_addr as u64, phy_addr as u64, mmu::user_page_flag());
            };

            return;
        }
    }

    panic!("failed to map virtual memory");
}
