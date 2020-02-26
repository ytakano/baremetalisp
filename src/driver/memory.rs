#[cfg(any(feature = "raspi3", feature = "raspi2"))]
pub const MMIO_BASE: u32 = 0x3F000000;

#[cfg(feature = "raspi4")]
pub const MMIO_BASE: u32 = 0xFE000000;

pub const GPFSEL0:   *mut u32 = (MMIO_BASE + 0x00200000) as *mut u32;
pub const GPFSEL1:   *mut u32 = (MMIO_BASE + 0x00200004) as *mut u32;
pub const GPFSEL2:   *mut u32 = (MMIO_BASE + 0x00200008) as *mut u32;
pub const GPFSEL3:   *mut u32 = (MMIO_BASE + 0x0020000C) as *mut u32;
pub const GPFSEL4:   *mut u32 = (MMIO_BASE + 0x00200010) as *mut u32;
pub const GPFSEL5:   *mut u32 = (MMIO_BASE + 0x00200014) as *mut u32;
pub const GPSET0:    *mut u32 = (MMIO_BASE + 0x0020001C) as *mut u32;
pub const GPSET1:    *mut u32 = (MMIO_BASE + 0x00200020) as *mut u32;
pub const GPCLR0:    *mut u32 = (MMIO_BASE + 0x00200028) as *mut u32;
pub const GPLEV0:    *mut u32 = (MMIO_BASE + 0x00200034) as *mut u32;
pub const GPLEV1:    *mut u32 = (MMIO_BASE + 0x00200038) as *mut u32;
pub const GPEDS0:    *mut u32 = (MMIO_BASE + 0x00200040) as *mut u32;
pub const GPEDS1:    *mut u32 = (MMIO_BASE + 0x00200044) as *mut u32;
pub const GPHEN0:    *mut u32 = (MMIO_BASE + 0x00200064) as *mut u32;
pub const GPHEN1:    *mut u32 = (MMIO_BASE + 0x00200068) as *mut u32;
pub const GPPUD:     *mut u32 = (MMIO_BASE + 0x00200094) as *mut u32;
pub const GPPUDCLK0: *mut u32 = (MMIO_BASE + 0x00200098) as *mut u32;
pub const GPPUDCLK1: *mut u32 = (MMIO_BASE + 0x0020009C) as *mut u32;

pub const AUX_ENABLE:     *mut u32 = (MMIO_BASE + 0x00215004) as *mut u32;
pub const AUX_MU_IO:      *mut u32 = (MMIO_BASE + 0x00215040) as *mut u32;
pub const AUX_MU_IER:     *mut u32 = (MMIO_BASE + 0x00215044) as *mut u32;
pub const AUX_MU_IIR:     *mut u32 = (MMIO_BASE + 0x00215048) as *mut u32;
pub const AUX_MU_LCR:     *mut u32 = (MMIO_BASE + 0x0021504C) as *mut u32;
pub const AUX_MU_MCR:     *mut u32 = (MMIO_BASE + 0x00215050) as *mut u32;
pub const AUX_MU_LSR:     *mut u32 = (MMIO_BASE + 0x00215054) as *mut u32;
pub const AUX_MU_MSRL:    *mut u32 = (MMIO_BASE + 0x00215058) as *mut u32;
pub const AUX_MU_SCRATCH: *mut u32 = (MMIO_BASE + 0x0021505C) as *mut u32;
pub const AUX_MU_CNTL:    *mut u32 = (MMIO_BASE + 0x00215060) as *mut u32;
pub const AUX_MU_STAT:    *mut u32 = (MMIO_BASE + 0x00215064) as *mut u32;
pub const AUX_MU_BAUD:    *mut u32 = (MMIO_BASE + 0x00215068) as *mut u32;