use core::ptr::{read_volatile, write_volatile};

pub(crate) mod cpu;
pub(crate) mod defs;
pub(crate) mod mbox;
pub(crate) mod memory;
pub(crate) mod mhu;
pub(crate) mod power;
pub(crate) mod psci;
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

pub(crate) fn init_platform_r_twi(_socid: SoCID, _use_rsb: bool) -> bool {
    // TODO
    false
}
