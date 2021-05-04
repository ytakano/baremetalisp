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

    crate::print::msg("map", "0");

    // disable interrupts
    let _mask = InterMask::new();

    let mut node = MCSNode::new();
    let mut lock = PAGER.lock(&mut node);

    if let GlobalVar::Having(pager) = &mut *lock {
        if let Some(phy_addr) = pager.alloc() {
            lock.unlock();
            if is_kern {
                crate::print::msg("map", "1");
                let mut ttbr1 = mmu::get_ttbr1();

                crate::print::hex64("phy_addr", phy_addr as u64);

                let vm_addr2 = vm_addr as u64 & !mmu::EL1_ADDR_OFFSET;
                crate::print::hex64("vm_addr2", vm_addr2);

                ttbr1.map(vm_addr2, phy_addr as u64, mmu::kernel_page_flag());

                if let Some(phy_addr2) = ttbr1.to_phy_addr(vm_addr2) {
                    crate::print::hex64("phy_addr2", phy_addr2);
                }
            } else {
                let mut ttbr0 = mmu::get_ttbr0();
                ttbr0.map(vm_addr as u64, phy_addr as u64, mmu::user_page_flag());
            };

            return;
        }
    }

    panic!("failed to map virtual memory");
}
