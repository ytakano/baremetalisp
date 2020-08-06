#[cfg(any(feature = "raspi3", feature = "raspi4"))]
use super::device::raspi::memory;

#[cfg(feature = "pine64")]
use super::device::allwinner::memory;

pub const DEVICE_MEM_START: u64 = memory::DEVICE_MEM_START;
pub const DEVICE_MEM_END: u64 = memory::DEVICE_MEM_END;

#[cfg(feature = "pine64")]
pub const CSS_SCP_COM_SHARED_MEM_BASE: u32 = memory::CSS_SCP_COM_SHARED_MEM_BASE;
