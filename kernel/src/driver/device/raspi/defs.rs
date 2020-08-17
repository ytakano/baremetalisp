#[cfg(feature = "raspi3")]
pub(crate) const SYSCNT_FRQ: u32 = 19200000;

#[cfg(feature = "raspi4")]
pub(crate) const SYSCNT_FRQ: u32 = 54000000;
