#[cfg(feature = "raspi3")]
pub(crate) const SYSCNT_FRQ: u32 = 19200000;

#[cfg(feature = "raspi4")]
pub(crate) const SYSCNT_FRQ: u32 = 54000000;

pub(crate) const MAX_PWR_LVL: u32 = 1;
