pub(in crate::driver) const SYSCNT_FRQ: u32 = 24000000;
pub(in crate::driver) const MAX_PWR_LVL: u8 = 2;
pub(in crate::driver) const MAX_RET_STATE: u8 = 1;
pub(in crate::driver) const MAX_OFF_STATE: u8 = 2;

pub(in crate::driver) const AXP803_CHIP_ID: u32 = 0x41;
pub(in crate::driver) const AXP805_CHIP_ID: u32 = 0x40;
pub(in crate::driver) const AXP_CHIP_ID: u32 = AXP803_CHIP_ID; // Allwinner A64

pub(in crate::driver) const CPU_PWR_LVL: u8 = 0;
pub(in crate::driver) const CLUSTER_PWR_LVL: u8 = 1;
pub(in crate::driver) const SYSTEM_PWR_LVL: u8 = 2;
