use core::default::Default;
use core::mem;
use synctools::mcs::{MCSLock, MCSNode};

const GIC_MAX_INTS: usize = 1020;
const NUM_INTS_PER_REG: usize = 32;

static GIC_GLOBAL: MCSLock<OptionGIC> = MCSLock::new(OptionGIC::UnInit);

#[derive(Default)]
pub struct GIC {
    gicc_base: usize,
    gicd_base: usize,
}

enum OptionGIC {
    Taked,
    Having(GIC),
    UnInit,
}

impl OptionGIC {
    fn take(&mut self) -> OptionGIC {
        mem::take(self)
    }
}

impl Default for OptionGIC {
    fn default() -> Self {
        OptionGIC::Taked
    }
}

pub fn init(gicc_base: usize, gicd_base: usize) {
    let mut node = MCSNode::new();
    let mut lock = GIC_GLOBAL.lock(&mut node);

    if let OptionGIC::UnInit = *lock {
        *lock = OptionGIC::Having(GIC {
            gicc_base,
            gicd_base,
        })
    } else {
        panic!("initialized twice");
    }
}

pub fn take() -> Option<GIC> {
    let mut node = MCSNode::new();
    let mut lock = GIC_GLOBAL.lock(&mut node);
    let gic = lock.take();
    lock.unlock();

    if let OptionGIC::Having(g) = gic {
        Some(g)
    } else {
        None
    }
}

impl Drop for GIC {
    fn drop(&mut self) {
        let mut node = MCSNode::new();
        let mut lock = GIC_GLOBAL.lock(&mut node);
        *lock = OptionGIC::Having(GIC {
            gicc_base: self.gicc_base,
            gicd_base: self.gicd_base,
        });
    }
}
