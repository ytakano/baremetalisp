#[cfg(feature = "raspi3")]
pub(in crate::driver) const SYSCNT_FRQ: u32 = 19200000;

#[cfg(feature = "raspi4")]
pub(in crate::driver) const SYSCNT_FRQ: u32 = 54000000;

pub(in crate::driver) const MAX_PWR_LVL: u8 = 1;
pub(in crate::driver) const MAX_RET_STATE: u8 = 1;
pub(in crate::driver) const MAX_OFF_STATE: u8 = 2;
