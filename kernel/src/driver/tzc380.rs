use crate::{global::GlobalVar, mmio::ReadWrite, out};
use synctools::mcs::{MCSLock, MCSNode};

const BUILD_CONFIG_AW_SHIFT: u32 = 8;
const BUILD_CONFIG_AW_MASK: u32 = 0x3f;
const BUILD_CONFIG_NR_SHIFT: u32 = 0;
const BUILD_CONFIG_NR_MASK: u32 = 0xf;

pub const TZC_ATTR_REGION_EN_MASK: u32 = 0x1;

const TZC_REGION_SIZE_SHIFT: u32 = 1;

pub const TZC_REGION_SIZE_32K: u32 = 0xe;
pub const TZC_REGION_SIZE_64K: u32 = 0xf;
pub const TZC_REGION_SIZE_128K: u32 = 0x10;
pub const TZC_REGION_SIZE_256K: u32 = 0x11;
pub const TZC_REGION_SIZE_512K: u32 = 0x12;
pub const TZC_REGION_SIZE_1M: u32 = 0x13;
pub const TZC_REGION_SIZE_2M: u32 = 0x14;
pub const TZC_REGION_SIZE_4M: u32 = 0x15;
pub const TZC_REGION_SIZE_8M: u32 = 0x16;
pub const TZC_REGION_SIZE_16M: u32 = 0x17;
pub const TZC_REGION_SIZE_32M: u32 = 0x18;
pub const TZC_REGION_SIZE_64M: u32 = 0x19;
pub const TZC_REGION_SIZE_128M: u32 = 0x1a;
pub const TZC_REGION_SIZE_256M: u32 = 0x1b;
pub const TZC_REGION_SIZE_512M: u32 = 0x1c;
pub const TZC_REGION_SIZE_1G: u32 = 0x1d;
pub const TZC_REGION_SIZE_2G: u32 = 0x1e;
pub const TZC_REGION_SIZE_4G: u32 = 0x1f;
pub const TZC_REGION_SIZE_8G: u32 = 0x20;
pub const TZC_REGION_SIZE_16G: u32 = 0x21;
pub const TZC_REGION_SIZE_32G: u32 = 0x22;
pub const TZC_REGION_SIZE_64G: u32 = 0x23;
pub const TZC_REGION_SIZE_128G: u32 = 0x24;
pub const TZC_REGION_SIZE_256G: u32 = 0x25;
pub const TZC_REGION_SIZE_512G: u32 = 0x26;
pub const TZC_REGION_SIZE_1T: u32 = 0x27;
pub const TZC_REGION_SIZE_2T: u32 = 0x28;
pub const TZC_REGION_SIZE_4T: u32 = 0x29;
pub const TZC_REGION_SIZE_8T: u32 = 0x2a;
pub const TZC_REGION_SIZE_16T: u32 = 0x2b;
pub const TZC_REGION_SIZE_32T: u32 = 0x2c;
pub const TZC_REGION_SIZE_64T: u32 = 0x2d;
pub const TZC_REGION_SIZE_128T: u32 = 0x2e;
pub const TZC_REGION_SIZE_256T: u32 = 0x2f;
pub const TZC_REGION_SIZE_512T: u32 = 0x30;
pub const TZC_REGION_SIZE_1P: u32 = 0x31;
pub const TZC_REGION_SIZE_2P: u32 = 0x32;
pub const TZC_REGION_SIZE_4P: u32 = 0x33;
pub const TZC_REGION_SIZE_8P: u32 = 0x34;
pub const TZC_REGION_SIZE_16P: u32 = 0x35;
pub const TZC_REGION_SIZE_32P: u32 = 0x36;
pub const TZC_REGION_SIZE_64P: u32 = 0x37;
pub const TZC_REGION_SIZE_128P: u32 = 0x38;
pub const TZC_REGION_SIZE_256P: u32 = 0x39;
pub const TZC_REGION_SIZE_512P: u32 = 0x3a;
pub const TZC_REGION_SIZE_1E: u32 = 0x3b;
pub const TZC_REGION_SIZE_2E: u32 = 0x3c;
pub const TZC_REGION_SIZE_4E: u32 = 0x3d;
pub const TZC_REGION_SIZE_8E: u32 = 0x3e;
pub const TZC_REGION_SIZE_16E: u32 = 0x3f;

const TZC_ATTR_SP_SHIFT: u32 = 28;
const TZC_ATTR_SP_MASK: u32 = 0b1111 << TZC_ATTR_SP_SHIFT;

const TZC_SP_NS_W: u32 = 1 << 0;
const TZC_SP_NS_R: u32 = 1 << 1;
const TZC_SP_S_W: u32 = 1 << 2;
const TZC_SP_S_R: u32 = 1 << 3;

pub const TZC_ATTR_SP_ALL: u32 =
    (TZC_SP_S_W | TZC_SP_S_R | TZC_SP_NS_W | TZC_SP_NS_R) << TZC_ATTR_SP_SHIFT;
pub const TZC_ATTR_SP_S_RW: u32 = (TZC_SP_S_W | TZC_SP_S_R) << TZC_ATTR_SP_SHIFT;
pub const TZC_ATTR_SP_NS_RW: u32 = (TZC_SP_NS_W | TZC_SP_NS_R) << TZC_ATTR_SP_SHIFT;

static TZC380_GLOBAL: MCSLock<GlobalVar<TZC380>> = MCSLock::new(GlobalVar::UnInit);

pub struct TZC380 {
    base: usize,
    addr_width: u32,
    num_regions: u32,
}

impl TZC380 {
    pub fn configure_region(&self, region: u8, base: usize, attr: u32) {
        if region as u32 >= self.num_regions {
            out::decimal("TZC region error", region as u64);
            return;
        }

        if region > 0 {
            self.setup_low(region).write((base & 0xffffffff) as u32);
            self.setup_high(region).write((base >> 32) as u32);
            self.attributes(region).write(attr);
        } else {
            self.attributes(region).write(attr & TZC_ATTR_SP_MASK);
        }
    }

    fn build_config(&self) -> ReadWrite<u32> {
        ReadWrite::new(self.base)
    }

    fn setup_low(&self, region: u8) -> ReadWrite<u32> {
        ReadWrite::new(self.base + 0x100 + region as usize * 0x10)
    }

    fn setup_high(&self, region: u8) -> ReadWrite<u32> {
        ReadWrite::new(self.base + 0x104 + region as usize * 0x10)
    }

    fn attributes(&self, region: u8) -> ReadWrite<u32> {
        ReadWrite::new(self.base + 0x108 + region as usize * 0x10)
    }
}

pub fn init(base: usize) {
    let tzc_build = ReadWrite::<u32>::new(base);
    let cfg = tzc_build.read();
    let addr_width = (((cfg >> BUILD_CONFIG_AW_SHIFT) & BUILD_CONFIG_AW_MASK) + 1) as u32;
    let num_regions = (((cfg >> BUILD_CONFIG_NR_SHIFT) & BUILD_CONFIG_NR_MASK) + 1) as u32;

    let tzc = TZC380 {
        base,
        addr_width,
        num_regions,
    };

    out::decimal("TZC Width", addr_width as u64);
    out::decimal("TZC #Regions", num_regions as u64);

    let mut node = MCSNode::new();
    let mut lock = TZC380_GLOBAL.lock(&mut node);

    if let GlobalVar::UnInit = *lock {
        *lock = GlobalVar::Having(tzc);
    } else {
        panic!("initialized twice");
    }
}

pub fn take() -> Option<TZC380> {
    let mut node = MCSNode::new();
    let mut lock = TZC380_GLOBAL.lock(&mut node);
    let tzc = lock.take();
    lock.unlock();

    if let GlobalVar::Having(t) = tzc {
        Some(t)
    } else {
        None
    }
}

pub fn attr_region_size(attr: u32) -> u32 {
    attr << TZC_REGION_SIZE_SHIFT
}
