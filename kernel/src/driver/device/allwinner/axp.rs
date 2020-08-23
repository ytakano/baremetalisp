use super::defs;
use super::rsb;
use crate::driver::uart;

pub const AXP803_HW_ADDR: u32 = 0x3a3;
pub const AXP803_RT_ADDR: u32 = 0x2d;

pub fn check_id() -> bool {
    let val = match read(0x03) {
        Some(v) => v,
        None => {
            return false;
        }
    };

    let val = val & 0xcf;
    if val != defs::AXP_CHIP_ID {
        return false;
    }

    uart::puts("PMIC: Found unknown PMIC 0x");
    uart::hex32(val);
    uart::puts("\n");

    true
}

pub(crate) fn read(reg: u32) -> Option<u32> {
    rsb::read(AXP803_RT_ADDR, reg)
}
