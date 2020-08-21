use core::ptr::{read_volatile, write_volatile};

use super::{init_platform_r_twi, read_soc_id, SoCID};

static mut PMIC: PMICType = PMICType::UNKNOWN;

enum PMICType {
    UNKNOWN,
    GenericH5,
    GenericA64,
    RefDesignH5, // regulators controlled by GPIO pins on port L
    AXP803RSB,   // PMIC connected via RSB on most A64 boards
}

fn get_pmic() -> PMICType {
    unsafe { read_volatile(&PMIC) }
}

fn set_pmic(pmic: PMICType) {
    unsafe {
        write_volatile(&mut PMIC, pmic);
    }
}

fn rsb_init() -> bool {
    // TODO
    false
}

pub(crate) fn init() {
    match read_soc_id() {
        SoCID::H5 => {
            set_pmic(PMICType::RefDesignH5);
        }
        SoCID::A64 => {
            if !init_platform_r_twi(SoCID::A64, true) {
                set_pmic(PMICType::GenericA64);
                return;
            }

            if !rsb_init() {
                set_pmic(PMICType::GenericA64);
                return;
            }

            set_pmic(PMICType::AXP803RSB);

            // TODO
            // axp_setup_regulators(fdt);
            // see https://github.com/ARM-software/arm-trusted-firmware/blob/007be5ecd14542a5da8533c14293faa1c44c3a7e/plat/allwinner/sun50i_a64/sunxi_power.c#L157
        }
        _ => {
            panic!("incompatible SoC ID");
        }
    }
}
