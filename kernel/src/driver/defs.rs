#[cfg(any(feature = "raspi3", feature = "raspi4"))]
use super::device::raspi::defs;

#[cfg(feature = "pine64")]
use super::device::allwinner::defs;

pub(crate) const SYSCNT_FRQ: u32 = defs::SYSCNT_FRQ;
