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
        uart::puts("PMIC: Found unknown PMIC 0x");
        uart::hex32(val);
        uart::puts("\n");
        return false;
    }

    true
}

pub(super) fn read(reg: u32) -> Option<u32> {
    rsb::read(AXP803_RT_ADDR, reg)
}

pub(super) fn write(reg: u32, val: u32) -> bool {
    rsb::write(AXP803_RT_ADDR, reg, val)
}

pub fn power_off() {
    // Set "power disable control" bit
    setbits(0x32, 1 << 7);
}

fn clrsetbits(reg: u32, clr_mask: u32, set_mask: u32) -> bool {
    let ret;
    match read(reg) {
        Some(x) => {
            ret = x;
        }
        None => {
            return false;
        }
    }
    let val = (ret & !clr_mask) | set_mask;
    write(reg, val)
}

fn setbits(reg: u32, set_mask: u32) -> bool {
    clrsetbits(reg, 0, set_mask)
}
