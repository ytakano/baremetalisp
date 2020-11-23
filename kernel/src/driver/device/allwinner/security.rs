use core::ptr::write_volatile;

use super::memory;

const R_PRCM_SEC_SWITCH_REG: u32 = 0x1d0;
const DMA_SEC_REG: u32 = 0x20;

fn spc_decport_set_reg(p: u32) -> u32 {
    memory::SUNXI_SPC_BASE + ((p) * 0x0c) + 0x8
}

pub(super) fn init() {
    // SPC setup: set all devices to non-secure
    for i in 0..6 {
        let ptr = spc_decport_set_reg(i) as *mut u32;
        unsafe {
            write_volatile(ptr, 0xff);
        }
    }

    unsafe {
        // set MBUS clocks, bus clocks (AXI/AHB/APB) and PLLs to non-secure
        let ptr = memory::SUNXI_CCU_SEC_SWITCH_REG as *mut u32;
        write_volatile(ptr, 0x7);

        // Set R_PRCM bus clocks to non-secure
        let ptr = (memory::SUNXI_R_PRCM_BASE + R_PRCM_SEC_SWITCH_REG) as *mut u32;
        write_volatile(ptr, 0x1);

        // Set all DMA channels (16 max.) to non-secure
        let ptr = (memory::SUNXI_DMA_BASE + DMA_SEC_REG) as *mut u32;
        write_volatile(ptr, 0xffff);
    }
}
