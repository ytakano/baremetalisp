use crate::{global::GlobalVar, mmio::MMIO, print_decimal};
use synctools::mcs::{MCSLock, MCSNode};

const BUILD_CONFIG_AW_SHIFT: u32 = 8;
const BUILD_CONFIG_AW_MASK: u32 = 0x3f;
const BUILD_CONFIG_NR_SHIFT: u32 = 0;
const BUILD_CONFIG_NR_MASK: u32 = 0xf;

static TZC380_GLOBAL: MCSLock<GlobalVar<TZC380>> = MCSLock::new(GlobalVar::UnInit);

struct TZC380 {
    base: usize,
    addr_width: u32,
    num_regions: u32,
}

impl TZC380 {
    fn build_config(&self) -> MMIO<u32> {
        MMIO::new(self.base as *mut u32)
    }
}

pub fn init(base: usize) {
    let tzc_build = MMIO::new(base as *mut u32);
    let cfg = tzc_build.read();
    let addr_width = (((cfg >> BUILD_CONFIG_AW_SHIFT) & BUILD_CONFIG_AW_MASK) + 1) as u32;
    let num_regions = (((cfg >> BUILD_CONFIG_NR_SHIFT) & BUILD_CONFIG_NR_MASK) + 1) as u32;

    let tzc = TZC380 {
        base,
        addr_width: 0,
        num_regions: 0,
    };

    print_decimal("TZC Width", addr_width as u64);
    print_decimal("TZC #Regions", num_regions as u64);

    let mut node = MCSNode::new();
    let mut lock = TZC380_GLOBAL.lock(&mut node);

    if let GlobalVar::UnInit = *lock {
        *lock = GlobalVar::Having(tzc);
    } else {
        panic!("initialized twice");
    }
}
