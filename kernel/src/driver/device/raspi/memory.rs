use crate::{mmio_rw, mmio_w};

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

const GPIO_BASE: usize = MMIO_BASE + 0x00200000;

mmio_rw!(GPIO_BASE         => pub(super) gpfsel0<u32>);
mmio_rw!(GPIO_BASE + 0x004 => pub(super) gpfsel1<u32>);
mmio_rw!(GPIO_BASE + 0x008 => pub(super) gpfsel2<u32>);
mmio_rw!(GPIO_BASE + 0x00c => pub(super) gpfsel3<u32>);
mmio_rw!(GPIO_BASE + 0x010 => pub(super) gpfsel4<u32>);
mmio_rw!(GPIO_BASE + 0x014 => pub(super) gpfsel5<u32>);
mmio_w! (GPIO_BASE + 0x01c => pub(super) gpset0<u32>);
mmio_w! (GPIO_BASE + 0x020 => pub(super) gpset1<u32>);
mmio_w! (GPIO_BASE + 0x028 => pub(super) gpclr0<u32>);
mmio_w! (GPIO_BASE + 0x02c => pub(super) gpclr1<u32>);
mmio_w! (GPIO_BASE + 0x034 => pub(super) gplev0<u32>);
mmio_w! (GPIO_BASE + 0x038 => pub(super) gplev1<u32>);
mmio_rw!(GPIO_BASE + 0x040 => pub(super) gpeds0<u32>);
mmio_rw!(GPIO_BASE + 0x044 => pub(super) gpeds1<u32>);
mmio_rw!(GPIO_BASE + 0x04c => pub(super) gpren0<u32>);
mmio_rw!(GPIO_BASE + 0x050 => pub(super) gpren1<u32>);
mmio_rw!(GPIO_BASE + 0x058 => pub(super) gpfen0<u32>);
mmio_rw!(GPIO_BASE + 0x05c => pub(super) gpfen1<u32>);
mmio_rw!(GPIO_BASE + 0x064 => pub(super) gphen0<u32>);
mmio_rw!(GPIO_BASE + 0x068 => pub(super) gphen1<u32>);
mmio_rw!(GPIO_BASE + 0x070 => pub(super) gplen0<u32>);
mmio_rw!(GPIO_BASE + 0x074 => pub(super) gplen1<u32>);
mmio_rw!(GPIO_BASE + 0x07c => pub(super) gparen0<u32>);
mmio_rw!(GPIO_BASE + 0x080 => pub(super) gparen1<u32>);
mmio_rw!(GPIO_BASE + 0x088 => pub(super) gpafen0<u32>);
mmio_rw!(GPIO_BASE + 0x08c => pub(super) gpafen1<u32>);
mmio_rw!(GPIO_BASE + 0x094 => pub(super) gppud<u32>);
mmio_rw!(GPIO_BASE + 0x098 => pub(super) gppudclk0<u32>);
mmio_rw!(GPIO_BASE + 0x09c => pub(super) gppudclk1<u32>);
