use core::ptr::write_volatile;

use super::cpu;
use super::memory;
use super::power;
use super::psci;
use super::security;
use super::{read_soc_id, SoCID};
use crate::driver::arm::gic;
use crate::{aarch64, print_msg};

pub(in crate::driver) fn early_platform_setup() {}

pub(in crate::driver) fn platform_setup() {
    cpu::disable_secondary_cpus(aarch64::cpu::mpidr_el1::get() as usize);

    // TODO
    // get device tree
    // see https://github.com/ARM-software/arm-trusted-firmware/blob/007be5ecd14542a5da8533c14293faa1c44c3a7e/plat/allwinner/common/sunxi_bl31_setup.c#L137-L147

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

    let soc_id = read_soc_id();
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
            print_msg("SoC", "Allwinner A64");
        }
        SoCID::H5 => {
            print_msg("SoC", "Allwinner H5");
        }
        SoCID::H6 => {
            print_msg("SoC", "Allwinner H6");
        }
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

    power::init();
    psci::init();
}
