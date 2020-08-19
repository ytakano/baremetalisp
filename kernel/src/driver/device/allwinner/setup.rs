use core::ptr::{read_volatile, write_volatile};

use super::memory;
use super::psci;
use super::security;
use crate::driver::arm::gic;
//use crate::driver::uart;

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
    // Configure the interrupt controller
    let driver_data = gic::v2::GICv2DriverData::new_gicd_gicc(
        memory::SUNXI_GICD_BASE as usize,
        memory::SUNXI_GICC_BASE as usize,
    );

    gic::v2::driver_init(&driver_data);
    gic::v2::distif_init();
    gic::v2::pcpu_distif_init();
    gic::v2::cpuif_enable();

    security::init();

    let soc_id = sunxi_read_soc_id();
    // On the A64 U-Boot's SPL sets the bus clocks to some conservative
    // values, to work around FEL mode instabilities with SRAM C accesses.
    // FEL mode is gone when we reach ATF, so bring the AHB1 bus
    // (the "main" bus) clock frequency back to the recommended 200MHz,
    // for improved performance.
    match &soc_id {
        SoCID::A64 => {
            let ptr = (memory::SUNXI_CCU_BASE + 0x54) as *mut u32;
            unsafe {
                write_volatile(ptr, 0x00003180);
            }
        }
        _ => (),
    }

    // U-Boot or the kernel don't setup AHB2, which leaves it at the
    // AHB1 frequency (200 MHz, see above). However Allwinner recommends
    // 300 MHz, for improved Ethernet and USB performance. Switch the
    // clock to use "PLL_PERIPH0 / 2".
    match &soc_id {
        SoCID::A64 | SoCID::H5 => {
            let ptr = (memory::SUNXI_CCU_BASE + 0x5c) as *mut u32;
            unsafe {
                write_volatile(ptr, 0x1);
            }
        }
        _ => (),
    }

    psci::init();
}
