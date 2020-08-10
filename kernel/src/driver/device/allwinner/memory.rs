pub const DEVICE_MEM_START: u64 = 0x01C00000;
pub const DEVICE_MEM_END: u64 = 0x01F10000;

pub const BL31_LIMIT: u32 = SUNXI_SRAM_A2_BASE + SUNXI_SRAM_A2_SIZE - SUNXI_SCP_SIZE;
pub const BL31_BASE: u32 = SUNXI_SRAM_A2_BASE + 0x4000;

pub const CSS_SCP_COM_SHARED_MEM_BASE: u32 = SUNXI_SRAM_A2_BASE + SUNXI_SRAM_A2_SIZE - 0x200;

pub const SUNXI_SRAM_A2_BASE: u32 = 0x00040000;
pub const SUNXI_SRAM_A2_SIZE: u32 = 0x00014000;
pub const SUNXI_SCP_BASE: u32 = BL31_LIMIT;
pub const SUNXI_SCP_SIZE: u32 = 0x4000;
pub const SUNXI_CPUCFG_BASE: u32 = 0x01700000;
pub const SUNXI_MSGBOX_BASE: u32 = 0x01c17000;
pub const SUNXI_UART0_BASE: u32 = 0x01c28000;
pub const SUNXI_R_PRCM_BASE: u32 = 0x01f01400;
pub const SUNXI_R_CPUCFG_BASE: u32 = 0x01f01c00;

pub const DRAM_BASE: u64 = 0x40000000;