#[cfg(any(feature = "raspi3", feature = "raspi4"))]
use super::device::raspi::defs;

#[cfg(feature = "pine64")]
use super::device::allwinner::defs;

pub const SYSCNT_FRQ: u32 = defs::SYSCNT_FRQ;
pub const MAX_PWR_LVL: u32 = defs::MAX_PWR_LVL;
