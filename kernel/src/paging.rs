use crate::global::GlobalVar;
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
