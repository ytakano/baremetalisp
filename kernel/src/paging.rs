use crate::{
    aarch64::{int::InterMask, mmu},
    allocator,
    global::GlobalVar,
    process::get_raw_id,
};
use memalloc::pager::PageManager;
use synctools::mcs::{MCSLock, MCSNode};

static PAGER: MCSLock<GlobalVar<PageManager>> = MCSLock::new(GlobalVar::UnInit);

pub enum FaultResult {
    Ok,
    StackOverflow,
    InvalidAccess,
}

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

pub fn map_user(vm_addr: usize, id: u8) -> FaultResult {
    if allocator::is_user_canary(id, vm_addr) {
        return FaultResult::StackOverflow;
    }

    if !allocator::is_user_mem(id, vm_addr) {
        return FaultResult::InvalidAccess;
    }

    map(vm_addr, vm_addr, false);
    FaultResult::Ok
}

pub fn fault(vm_addr: usize) -> FaultResult {
    if allocator::is_kern_mem(vm_addr) {
        map(vm_addr, vm_addr, true);
        FaultResult::Ok
    } else if let Some(id) = get_raw_id() {
        if allocator::is_user_canary(id, vm_addr) {
            return FaultResult::StackOverflow;
        }

        if !allocator::is_user_mem(id, vm_addr) {
            return FaultResult::InvalidAccess;
        }

        map(vm_addr, vm_addr, false);
        FaultResult::Ok
    } else {
        FaultResult::InvalidAccess
    }
}

fn map(start: usize, end: usize, is_kern: bool) {
    // disable interrupts
    let _mask = InterMask::new();

    let mut node = MCSNode::new();
    let mut lock = PAGER.lock(&mut node);

    if let GlobalVar::Having(pager) = &mut *lock {
        for vm_addr in (start..=end).step_by(mmu::PAGESIZE as usize) {
            if let Some(phy_addr) = pager.alloc() {
                lock.unlock();
                if is_kern {
                    let mut ttbr1 = mmu::get_ttbr1();
                    if let None = ttbr1.to_phy_addr(vm_addr as u64) {
                        ttbr1.map(vm_addr as u64, phy_addr as u64, mmu::kernel_page_flag());
                    }
                    mmu::tlb_flush_addr(vm_addr);
                } else {
                    let mut ttbr0 = mmu::get_ttbr0();
                    if let None = ttbr0.to_phy_addr(vm_addr as u64) {
                        ttbr0.map(vm_addr as u64, phy_addr as u64, mmu::user_page_flag());
                    }
                };

                return;
            }
        }
    }

    panic!("failed to map virtual memory");
}
