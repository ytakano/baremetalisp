use register::{mmio::*, register_structs};

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

register_structs! {
    #[allow(non_snake_case)]
    pub(super) GPIORegisters {
        (0x000 => pub(super) GPFSEL0: ReadWrite<u32>),
        (0x004 => pub(super) GPFSEL1: ReadWrite<u32>),
        (0x008 => pub(super) GPFSEL2: ReadWrite<u32>),
        (0x00c => pub(super) GPFSEL3: ReadWrite<u32>),
        (0x010 => pub(super) GPFSEL4: ReadWrite<u32>),
        (0x014 => pub(super) GPFSEL5: ReadWrite<u32>),

        (0x01c => pub(super) GPSET0: WriteOnly<u32>),
        (0x020 => pub(super) GPSET1: WriteOnly<u32>),

        (0x028 => pub(super) GPCLR0: WriteOnly<u32>),
        (0x02c => pub(super) GPCLR1: WriteOnly<u32>),

        (0x034 => pub(super) GPLEV0: ReadOnly<u32>),
        (0x038 => pub(super) GPLEV1: ReadOnly<u32>),

        (0x040 => pub(super) GPEDS0: ReadWrite<u32>),
        (0x044 => pub(super) GPEDS1: ReadWrite<u32>),

        (0x04c => pub(super) GPREN0: ReadWrite<u32>),
        (0x050 => pub(super) GPREN1: ReadWrite<u32>),

        (0x058 => pub(super) GPFEN0: ReadWrite<u32>),
        (0x05c => pub(super) GPFEN1: ReadWrite<u32>),

        (0x064 => pub(super) GPHEN0: ReadWrite<u32>),
        (0x068 => pub(super) GPHEN1: ReadWrite<u32>),

        (0x070 => pub(super) GPLEN0: ReadWrite<u32>),
        (0x074 => pub(super) GPLEN1: ReadWrite<u32>),

        (0x07c => pub(super) GPAREN0: ReadWrite<u32>),
        (0x080 => pub(super) GPAREN1: ReadWrite<u32>),

        (0x088 => pub(super) GPAFEN0: ReadWrite<u32>),
        (0x08c => pub(super) GPAFEN1: ReadWrite<u32>),

        (0x094 => pub(super) GPPUD: ReadWrite<u32>),
        (0x098 => pub(super) GPPUDCLK0: ReadWrite<u32>),
        (0x09c => pub(super) GPPUDCLK1: ReadWrite<u32>),

        (0x0a4 => @END),
    }
}

pub(super) fn gpio_registers() -> *const GPIORegisters {
    (MMIO_BASE + 0x00200000) as *const GPIORegisters
}
