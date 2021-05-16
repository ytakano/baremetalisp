#[cfg(any(feature = "raspi3", feature = "raspi4"))]
use super::device::raspi::memory;

#[cfg(feature = "pine64")]
use super::device::allwinner::memory;

pub const DEVICE_MEM_START: u64 = memory::DEVICE_MEM_START;
pub const DEVICE_MEM_END: u64 = memory::DEVICE_MEM_END;
pub const SRAM_START: u64 = memory::SRAM_START;
pub const SRAM_END: u64 = memory::SRAM_END;
pub const ROM_START: u64 = memory::ROM_START;
pub const ROM_END: u64 = memory::ROM_END;
pub const DRAM_BASE: u64 = memory::DRAM_BASE;
