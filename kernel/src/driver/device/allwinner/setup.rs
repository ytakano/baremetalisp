use core::ptr::{read_volatile, write_volatile};

use super::memory;
use super::psci;
use crate::driver::uart;

const SUNXI_SOC_A64: u32 = 0x1689;
const SUNXI_SOC_H5: u32 = 0x1718;
const SUNXI_SOC_H6: u32 = 0x1728;

enum SoCID {
    A64,
    H5,
    H6,
}

fn sunxi_read_soc_id() -> SoCID {
    let ver_reg = (memory::SUNXI_SYSCON_BASE + 0x24) as *mut u32;

    let ver;
    unsafe {
        let req = read_volatile(ver_reg);

        // Set bit 15 to prepare for the SOCID read.
        write_volatile(ver_reg, req | (1 << 15));

        ver = read_volatile(ver_reg);

        // deactivate the SOCID access again
        write_volatile(ver_reg, req & !(1 << 15));
    }

    match ver >> 16 {
        SUNXI_SOC_A64 => SoCID::A64,
        SUNXI_SOC_H5 => SoCID::H5,
        SUNXI_SOC_H6 => SoCID::H6,
        _ => {
            panic!("unkown SoC ID");
        }
    }
}

pub fn platform_setup() {
    psci::init();
}
