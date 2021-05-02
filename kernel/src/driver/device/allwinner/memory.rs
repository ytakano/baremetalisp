// Allwinner A64

pub(in crate::driver) const DEVICE_MEM_START: u64 = 0x01000000;
pub(in crate::driver) const DEVICE_MEM_END: u64 = 0x02000000;
pub(in crate::driver) const ROM_START: u64 = 0x00000000;
pub(in crate::driver) const ROM_END: u64 = 0x00010000;
pub(in crate::driver) const SRAM_START: u64 = 0x00010000;
pub(in crate::driver) const SRAM_END: u64 = 0x00054000;

pub(in crate::driver) const CSS_SCP_COM_SHARED_MEM_BASE: u32 =
    SUNXI_SRAM_A2_BASE + SUNXI_SRAM_A2_SIZE - 0x200;

// Memory regions
pub(in crate::driver) const SUNXI_ROM_BASE: u32 = 0x00000000;
pub(in crate::driver) const SUNXI_ROM_SIZE: u32 = 0x00010000;
pub(in crate::driver) const SUNXI_SRAM_BASE: u32 = 0x00010000;
pub(in crate::driver) const SUNXI_SRAM_SIZE: u32 = 0x00044000;
pub(in crate::driver) const SUNXI_SRAM_A1_BASE: u32 = 0x00010000;
pub(in crate::driver) const SUNXI_SRAM_A1_SIZE: u32 = 0x00008000;
pub(in crate::driver) const SUNXI_SRAM_A2_BASE: u32 = 0x00040000;
pub(in crate::driver) const SUNXI_SRAM_A2_SIZE: u32 = 0x00014000;
pub(in crate::driver) const SUNXI_SRAM_C_BASE: u32 = 0x00018000;
pub(in crate::driver) const SUNXI_SRAM_C_SIZE: u32 = 0x0001c000;

pub(in crate::driver) const SUNXI_SCP_BASE: u32 =
    SUNXI_SRAM_A2_BASE + SUNXI_SRAM_A2_SIZE - SUNXI_SCP_SIZE;
pub(in crate::driver) const SUNXI_SCP_SIZE: u32 = 0x4000;

// Memory-mapped devices
pub(in crate::driver) const SUNXI_CPU_MBIST_BASE: u32 = 0x01502000;
pub(in crate::driver) const SUNXI_CPUCFG_BASE: u32 = 0x01700000;
pub(in crate::driver) const SUNXI_GENER_CTRL_REG0: u32 = 0x01700028;
pub(in crate::driver) const SUNXI_SYSCON_BASE: u32 = 0x01c00000;
pub(in crate::driver) const SUNXI_DMA_BASE: u32 = 0x01c02000;
pub(in crate::driver) const SUNXI_KEYMEM_BASE: u32 = 0x01c0b000;
pub(in crate::driver) const SUNXI_SMHC0_BASE: u32 = 0x01c0f000;
pub(in crate::driver) const SUNXI_SMHC1_BASE: u32 = 0x01c10000;
pub(in crate::driver) const SUNXI_SMHC2_BASE: u32 = 0x01c11000;
pub(in crate::driver) const SUNXI_SID_BASE: u32 = 0x01c14000;
pub(in crate::driver) const SUNXI_MSGBOX_BASE: u32 = 0x01c17000;
pub(in crate::driver) const SUNXI_SPINLOCK_BASE: u32 = 0x01c18000;
pub(in crate::driver) const SUNXI_SMC_BASE: u32 = 0x01c1e000;
pub(in crate::driver) const SUNXI_CCU_BASE: u32 = 0x01c20000;
pub(in crate::driver) const SUNXI_CCU_SEC_SWITCH_REG: u32 = SUNXI_CCU_BASE + 0x2f0;
pub(in crate::driver) const SUNXI_PIO_BASE: u32 = 0x01c20800;
pub(in crate::driver) const SUNXI_TIMER_BASE: u32 = 0x01c20c00;
pub(in crate::driver) const SUNXI_WDOG_BASE: u32 = 0x01c20ca0;
pub(in crate::driver) const SUNXI_SPC_BASE: u32 = 0x01c23400;
pub(in crate::driver) const SUNXI_THS_BASE: u32 = 0x01c25000;
pub(in crate::driver) const SUNXI_UART0_BASE: u32 = 0x01c28000;
pub(in crate::driver) const SUNXI_UART1_BASE: u32 = 0x01c28400;
pub(in crate::driver) const SUNXI_UART2_BASE: u32 = 0x01c28800;
pub(in crate::driver) const SUNXI_UART3_BASE: u32 = 0x01c28c00;
pub(in crate::driver) const SUNXI_I2C0_BASE: u32 = 0x01c2ac00;
pub(in crate::driver) const SUNXI_I2C1_BASE: u32 = 0x01c2b000;
pub(in crate::driver) const SUNXI_I2C2_BASE: u32 = 0x01c2b400;
pub(in crate::driver) const SUNXI_DRAMCOM_BASE: u32 = 0x01c62000;
pub(in crate::driver) const SUNXI_DRAMCTL_BASE: u32 = 0x01c63000;
pub(in crate::driver) const SUNXI_DRAMPHY_BASE: u32 = 0x01c65000;
pub(in crate::driver) const SUNXI_SPI0_BASE: u32 = 0x01c68000;
pub(in crate::driver) const SUNXI_SPI1_BASE: u32 = 0x01c69000;
pub(in crate::driver) const SUNXI_SCU_BASE: u32 = 0x01c80000;
pub(in crate::driver) const SUNXI_GICD_BASE: u32 = 0x01c81000;
pub(in crate::driver) const SUNXI_GICC_BASE: u32 = 0x01c82000;
pub(in crate::driver) const SUNXI_RTC_BASE: u32 = 0x01f00000;
pub(in crate::driver) const SUNXI_R_TIMER_BASE: u32 = 0x01f00800;
pub(in crate::driver) const SUNXI_R_INTC_BASE: u32 = 0x01f00c00;
pub(in crate::driver) const SUNXI_R_WDOG_BASE: u32 = 0x01f01000;
pub(in crate::driver) const SUNXI_R_PRCM_BASE: u32 = 0x01f01400;
pub(in crate::driver) const SUNXI_R_TWD_BASE: u32 = 0x01f01800;
pub(in crate::driver) const SUNXI_R_CPUCFG_BASE: u32 = 0x01f01c00;
pub(in crate::driver) const SUNXI_R_CIR_BASE: u32 = 0x01f02000;
pub(in crate::driver) const SUNXI_R_I2C_BASE: u32 = 0x01f02400;
pub(in crate::driver) const SUNXI_R_UART_BASE: u32 = 0x01f02800;
pub(in crate::driver) const SUNXI_R_PIO_BASE: u32 = 0x01f02c00;
pub(in crate::driver) const SUNXI_R_RSB_BASE: u32 = 0x01f03400;
pub(in crate::driver) const SUNXI_R_PWM_BASE: u32 = 0x01f03800;

pub(in crate::driver) const DRAM_BASE: u64 = 0x40000000;
