use core::ptr::{read_volatile, write_volatile};

use crate::bits;

pub(crate) mod axp;
pub(crate) mod cpu;
pub(crate) mod defs;
pub(crate) mod mbox;
pub(crate) mod memory;
pub(crate) mod mhu;
pub(crate) mod power;
pub(crate) mod psci;
pub(crate) mod rsb;
pub(crate) mod security;
pub(crate) mod setup;
pub(crate) mod topology;
pub(crate) mod uart;

const SUNXI_SOC_A64: u32 = 0x1689;
const SUNXI_SOC_H5: u32 = 0x1718;
const SUNXI_SOC_H6: u32 = 0x1728;

pub(crate) enum SoCID {
    A64,
    H5,
    H6,
}

pub(crate) fn read_soc_id() -> SoCID {
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

pub(crate) fn init_platform_r_twi(socid: SoCID, use_rsb: bool) -> bool {
    let pin_func: u32;
    let device_bit: u32;
    let reset_offset: usize;
    match socid {
        SoCID::H5 => {
            if use_rsb {
                return false;
            }
            pin_func = 0x22;
            device_bit = 1 << 16;
            reset_offset = 0xb0;
        }
        SoCID::H6 => {
            if use_rsb {
                return false;
            }
            pin_func = 0x33;
            device_bit = 1 << 16;
            reset_offset = 0x19c;
        }
        SoCID::A64 => {
            pin_func = if use_rsb { 0x22 } else { 0x33 };
            device_bit = if use_rsb { 1 << 3 } else { 1 << 6 };
            reset_offset = 0xb0;
        }
    }

    // un-gate R_PIO clock
    match socid {
        SoCID::H6 => {
            let ptr = (memory::SUNXI_R_PRCM_BASE + 0x28) as *mut u32;
            bits::bit_set32(ptr, 0);
        }
        _ => (),
    }

    // switch pins PL0 and PL1 to the desired function
    let ptr = memory::SUNXI_R_PIO_BASE as *mut u32;
    bits::clrsetbits_32(ptr, 0xff, pin_func);

    // level 2 drive strength
    let ptr = (memory::SUNXI_R_PIO_BASE + 0x14) as *mut u32;
    bits::clrsetbits_32(ptr, 0x0f, 0x0a);

    // set both pins to pull-up
    let ptr = (memory::SUNXI_R_PIO_BASE + 0x1c) as *mut u32;
    bits::clrsetbits_32(ptr, 0x0f, 0x05);

    // un-gate clock
    match socid {
        SoCID::H6 => {
            let ptr = (memory::SUNXI_R_PRCM_BASE + 0x19c) as *mut u32;
            bits::setbits_32(ptr, device_bit | 1);
        }
        _ => {
            let ptr = (memory::SUNXI_R_PRCM_BASE + 0x28) as *mut u32;
            bits::setbits_32(ptr, device_bit);
        }
    }

    // assert, then de-assert reset of I2C/RSB controller
    let ptr = (memory::SUNXI_R_PRCM_BASE as usize + reset_offset) as *mut u32;
    bits::clrbits_32(ptr, device_bit);
    bits::setbits_32(ptr, device_bit);

    true
}
