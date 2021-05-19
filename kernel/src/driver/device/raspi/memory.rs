// https://wiki.osdev.org/Raspberry_Pi_4

//-----------------------------------------------------------------------------
// Raspberry Pi 3
#[cfg(feature = "raspi3")]
mod raspi {
    pub(super) const MMIO_BASE: usize = 0x3F000000;
    pub(super) const DEVICE_MEM_START: u64 = 0x3C000000;
    pub(super) const DEVICE_MEM_END: u64 = 0x40000000;
}
//-----------------------------------------------------------------------------

//-----------------------------------------------------------------------------
// Raspberry Pi 4
#[cfg(feature = "raspi4")]
mod raspi {
    pub(super) const MMIO_BASE: usize = 0xFE000000;
    pub(super) const DEVICE_MEM_START: u64 = 0x0fd000000; // maybe...
    pub(super) const DEVICE_MEM_END: u64 = 0x100000000; // maybe...
}
//-----------------------------------------------------------------------------

pub(in crate::driver) const SRAM_START: u64 = 0;
pub(in crate::driver) const SRAM_END: u64 = 0;
pub(in crate::driver) const ROM_START: u64 = 0;
pub(in crate::driver) const ROM_END: u64 = 0;

pub(in crate::driver) const MMIO_BASE: usize = raspi::MMIO_BASE;
pub(in crate::driver) const DEVICE_MEM_START: u64 = raspi::DEVICE_MEM_START;
pub(in crate::driver) const DEVICE_MEM_END: u64 = raspi::DEVICE_MEM_END;

pub(super) const GPFSEL0: *mut u32 = (MMIO_BASE + 0x00200000) as *mut u32;
pub(super) const GPFSEL1: *mut u32 = (MMIO_BASE + 0x00200004) as *mut u32;
pub(super) const GPFSEL2: *mut u32 = (MMIO_BASE + 0x00200008) as *mut u32;
pub(super) const GPFSEL3: *mut u32 = (MMIO_BASE + 0x0020000C) as *mut u32;
pub(super) const GPFSEL4: *mut u32 = (MMIO_BASE + 0x00200010) as *mut u32;
pub(super) const GPFSEL5: *mut u32 = (MMIO_BASE + 0x00200014) as *mut u32;
pub(super) const GPSET0: *mut u32 = (MMIO_BASE + 0x0020001C) as *mut u32;
pub(super) const GPSET1: *mut u32 = (MMIO_BASE + 0x00200020) as *mut u32;
pub(super) const GPCLR0: *mut u32 = (MMIO_BASE + 0x00200028) as *mut u32;
pub(super) const GPLEV0: *mut u32 = (MMIO_BASE + 0x00200034) as *mut u32;
pub(super) const GPLEV1: *mut u32 = (MMIO_BASE + 0x00200038) as *mut u32;
pub(super) const GPEDS0: *mut u32 = (MMIO_BASE + 0x00200040) as *mut u32;
pub(super) const GPEDS1: *mut u32 = (MMIO_BASE + 0x00200044) as *mut u32;
pub(super) const GPHEN0: *mut u32 = (MMIO_BASE + 0x00200064) as *mut u32;
pub(super) const GPHEN1: *mut u32 = (MMIO_BASE + 0x00200068) as *mut u32;
pub(super) const GPPUD: *mut u32 = (MMIO_BASE + 0x00200094) as *mut u32;
pub(super) const GPPUDCLK0: *mut u32 = (MMIO_BASE + 0x00200098) as *mut u32;
pub(super) const GPPUDCLK1: *mut u32 = (MMIO_BASE + 0x0020009C) as *mut u32;

pub(super) const AUX_ENABLE: *mut u32 = (MMIO_BASE + 0x00215004) as *mut u32;
pub(super) const AUX_MU_IO: *mut u32 = (MMIO_BASE + 0x00215040) as *mut u32;
pub(super) const AUX_MU_IER: *mut u32 = (MMIO_BASE + 0x00215044) as *mut u32;
pub(super) const AUX_MU_IIR: *mut u32 = (MMIO_BASE + 0x00215048) as *mut u32;
pub(super) const AUX_MU_LCR: *mut u32 = (MMIO_BASE + 0x0021504C) as *mut u32;
pub(super) const AUX_MU_MCR: *mut u32 = (MMIO_BASE + 0x00215050) as *mut u32;
pub(super) const AUX_MU_LSR: *mut u32 = (MMIO_BASE + 0x00215054) as *mut u32;
pub(super) const AUX_MU_MSRL: *mut u32 = (MMIO_BASE + 0x00215058) as *mut u32;
pub(super) const AUX_MU_SCRATCH: *mut u32 = (MMIO_BASE + 0x0021505C) as *mut u32;
pub(super) const AUX_MU_CNTL: *mut u32 = (MMIO_BASE + 0x00215060) as *mut u32;
pub(super) const AUX_MU_STAT: *mut u32 = (MMIO_BASE + 0x00215064) as *mut u32;
pub(super) const AUX_MU_BAUD: *mut u32 = (MMIO_BASE + 0x00215068) as *mut u32;

pub(in crate::driver) const DRAM_BASE: u64 = 0;
