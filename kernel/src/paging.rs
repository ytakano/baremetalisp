use crate::{aarch64::mmu, allocator, cpuint, global::GlobalVar, process::get_raw_id};
use memac::pager::PageManager;
use synctools::mcs::{MCSLock, MCSNode};

static PAGER: MCSLock<GlobalVar<PageManager>> = MCSLock::new(GlobalVar::UnInit);

#[derive(PartialEq, Eq)]
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

fn map_user(vm_addr: usize, id: u8) -> FaultResult {
    let vm_addr = vm_addr & memac::MASK;

    if allocator::is_user_canary(id, vm_addr) {
        return FaultResult::StackOverflow;
    }

    if !allocator::is_user_mem(id, vm_addr) {
        return FaultResult::InvalidAccess;
    }

    map(vm_addr, vm_addr, false);
    FaultResult::Ok
}

pub fn unmap_user(start: usize, end: usize, id: u8) {
    let start = start & memac::MASK;
    let end = end & memac::MASK;

    for addr in (start..end).step_by(memac::ALIGNMENT) {
        if allocator::is_user_canary(id, addr) {
            return;
        }

        if !allocator::is_user_mem(id, addr) {
            return;
        }
    }

    unmap(start, end, false);
}

pub fn unmap_user_all(id: u8) {
    let (start, end) = allocator::user_mem(id);
    unmap(start, end, false);
}

pub fn fault(vm_addr: usize) -> FaultResult {
    let vm_addr = vm_addr & memac::MASK;

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

pub fn map_canary() {
    if let Some(id) = get_raw_id() {
        let canary = allocator::user_canary(id) as usize;
        map(canary, canary, false);
    }
}

fn unmap(start: usize, end: usize, is_kern: bool) {
    // disable interrupts
    let _mask = cpuint::mask();

    let mut node = MCSNode::new();
    let mut lock = PAGER.lock(&mut node);

    let mut ttbr = if is_kern {
        mmu::get_ttbr1()
    } else {
        mmu::get_ttbr0()
    };

    if let GlobalVar::Having(pager) = &mut *lock {
        for vm_addr in (start..=end).step_by(memac::ALIGNMENT) {
            if let Some(phy_addr) = ttbr.to_phy_addr(vm_addr as u64) {
                pager.free(phy_addr as usize);
                ttbr.unmap(vm_addr as u64);
            }
        }

        if start == end {
            mmu::tlb_flush_addr(start);
        } else {
            mmu::tlb_flush_all();
        }

        return;
    }
}

fn map(start: usize, end: usize, is_kern: bool) {
    // disable interrupts
    let _mask = cpuint::mask();

    let mut node = MCSNode::new();
    let mut lock = PAGER.lock(&mut node);

    let (mut ttbr, flag) = if is_kern {
        (mmu::get_ttbr1(), mmu::kernel_page_flag())
    } else {
        (mmu::get_ttbr0(), mmu::user_page_flag())
    };

    if let GlobalVar::Having(pager) = &mut *lock {
        for vm_addr in (start..=end).step_by(memac::ALIGNMENT) {
            if ttbr.to_phy_addr(vm_addr as u64).is_none() {
                if let Some(phy_addr) = pager.alloc() {
                    ttbr.map(vm_addr as u64, phy_addr as u64, flag);
                } else {
                    panic!("exhausted memory");
                }
            } else {
                mmu::tlb_flush_addr(vm_addr);
            }
        }

        if start == end {
            mmu::tlb_flush_addr(start);
        } else {
            mmu::tlb_flush_all();
        }

        return;
    }
    lock.unlock();

    panic!("failed to map virtual memory");
}
