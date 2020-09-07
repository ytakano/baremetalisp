use core::ptr::{read_volatile, write_volatile};

use super::{axp, cpu, memory, rsb};
use super::{init_platform_r_twi, read_soc_id, SoCID};
use crate::aarch64;
use crate::bits;
use crate::driver::arm::{gic, scpi};
use crate::driver::{delays, uart};
use crate::print_msg;

static mut PMIC: PMICType = PMICType::UNKNOWN;

const SUNXI_WDOG0_CTRL_REG: *mut u32 = (memory::SUNXI_R_WDOG_BASE + 0x0010) as *mut u32;
const SUNXI_WDOG0_CFG_REG: *mut u32 = (memory::SUNXI_R_WDOG_BASE + 0x0014) as *mut u32;
const SUNXI_WDOG0_MODE_REG: *mut u32 = (memory::SUNXI_R_WDOG_BASE + 0x0018) as *mut u32;

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

pub(crate) fn init() {
    match read_soc_id() {
        SoCID::H5 => {
            set_pmic(PMICType::RefDesignH5);
            print_msg("PMIC", "RefDesignH5");
        }
        SoCID::A64 => {
            if !init_platform_r_twi(SoCID::A64, true) {
                set_pmic(PMICType::GenericA64);
                print_msg("PMIC", "GenericA64");
                return;
            }

            if !rsb::init() {
                set_pmic(PMICType::GenericA64);
                print_msg("PMIC", "GenericA64");
                return;
            }

            set_pmic(PMICType::AXP803RSB);
            print_msg("PMIC", "AXP803RSB");

            // TODO
            // axp_setup_regulators(fdt);
            // see https://github.com/ARM-software/arm-trusted-firmware/blob/007be5ecd14542a5da8533c14293faa1c44c3a7e/plat/allwinner/sun50i_a64/sunxi_power.c#L157
        }
        _ => {
            panic!("incompatible SoC ID");
        }
    }
}

pub fn system_off() {
    gic::v2::cpuif_disable();

    if cpu::scpi_available() {
        // Send the power down request to the SCP
        match scpi::sys_power_state(scpi::ScpiSystemState::Shutdown) {
            scpi::ScpiResult::Ok => (),
            ret => {
                uart::puts("error: PSCI: SCPI shutdown failed: ");
                uart::decimal(ret as u64);
                uart::puts("\n");
            }
        }
    }

    // Turn off all secondary CPUs
    cpu::disable_secondary_cpus(aarch64::cpu::mpidr_el1::get() as usize);

    uart::puts("PSCI: Turning off system\n");
    power_down();

    delays::wait_microsec(1000);
    uart::puts("error: PSCI: Cannot turn off system, halting\n");
    aarch64::cpu::wait_interrupt();
    panic!("failed shutdown");
}

pub fn power_down() {
    match get_pmic() {
        PMICType::GenericH5 => {
            // Turn off as many peripherals and clocks as we can.
            sunxi_turn_off_soc(SoCID::H5);
            // Turn off the pin controller now.
            let ptr = (memory::SUNXI_CCU_BASE + 0x68) as *mut u32;
            unsafe { write_volatile(ptr, 0) };
        }
        PMICType::GenericA64 => {
            // Turn off as many peripherals and clocks as we can.
            sunxi_turn_off_soc(SoCID::A64);
            // Turn off the pin controller now.
            let ptr = (memory::SUNXI_CCU_BASE + 0x68) as *mut u32;
            unsafe { write_volatile(ptr, 0) };
        }
        PMICType::RefDesignH5 => {
            sunxi_turn_off_soc(SoCID::H5);

            // TODO
            // Switch PL pins to power off the board:
            // - PL5 (VCC_IO) -> high
            // - PL8 (PWR-STB = CPU power supply) -> low
            // - PL9 (PWR-DRAM) ->low
            // - PL10 (power LED) -> low
            // Note: Clearing PL8 will reset the board, so keep it up.
            // sunxi_set_gpio_out('L', 5, 1);
            // sunxi_set_gpio_out('L', 9, 0);
            // sunxi_set_gpio_out('L', 10, 0);

            let ptr = (memory::SUNXI_CCU_BASE + 0x68) as *mut u32;
            unsafe { write_volatile(ptr, 0) };
        }
        PMICType::AXP803RSB => {
            // (Re-)init RSB in case the rich OS has disabled it.
            init_platform_r_twi(SoCID::A64, true);
            rsb::init();
            axp::power_off();
        }
        _ => (),
    }
}

/// On boards without a proper PMIC we struggle to turn off the system properly.
/// Try to turn off as much off the system as we can, to reduce power
/// consumption. This should be entered with only one core running and SMP
/// disabled.
/// This function only cares about peripherals.
fn sunxi_turn_off_soc(socid: SoCID) {
    // Turn off most peripherals, most importantly DRAM users.
    // Keep DRAM controller running for now.
    let ptr = (memory::SUNXI_CCU_BASE + 0x2c0) as *mut u32;
    bits::clrbits_32(ptr, !(1 << 14));
    let ptr = (memory::SUNXI_CCU_BASE + 0x60) as *mut u32;
    bits::clrbits_32(ptr, !(1 << 14));

    // Contains msgbox (bit 21) and spinlock (bit 22)
    unsafe {
        let ptr = (memory::SUNXI_CCU_BASE + 0x2c4) as *mut u32;
        write_volatile(ptr, 0);
        let ptr = (memory::SUNXI_CCU_BASE + 0x64) as *mut u32;
        write_volatile(ptr, 0);
        let ptr = (memory::SUNXI_CCU_BASE + 0x2c8) as *mut u32;
        write_volatile(ptr, 0);
    }

    // Keep PIO controller running for now.
    let ptr = (memory::SUNXI_CCU_BASE + 0x68) as *mut u32;
    bits::clrbits_32(ptr, !(1 << 5));
    let ptr = (memory::SUNXI_CCU_BASE + 0x2d0) as *mut u32;
    unsafe { write_volatile(ptr, 0) };

    // Contains UART0 (bit 16)
    let ptr = (memory::SUNXI_CCU_BASE + 0x2d8) as *mut u32;
    unsafe { write_volatile(ptr, 0) };
    let ptr = (memory::SUNXI_CCU_BASE + 0x6c) as *mut u32;
    unsafe { write_volatile(ptr, 0) };
    let ptr = (memory::SUNXI_CCU_BASE + 0x70) as *mut u32;
    unsafe { write_volatile(ptr, 0) };

    // Turn off DRAM controller.
    let ptr = (memory::SUNXI_CCU_BASE + 0x2c0) as *mut u32;
    bits::clrbits_32(ptr, 1 << 14);
    let ptr = (memory::SUNXI_CCU_BASE + 0x60) as *mut u32;
    bits::clrbits_32(ptr, 1 << 14);

    // Migrate CPU and bus clocks away from the PLLs.
    // AHB1: use OSC24M/1, APB1 = AHB1 / 2
    let ptr = (memory::SUNXI_CCU_BASE + 0x54) as *mut u32;
    unsafe { write_volatile(ptr, 0x1000) };
    // APB2: use OSC24M
    let ptr = (memory::SUNXI_CCU_BASE + 0x58) as *mut u32;
    unsafe { write_volatile(ptr, 0x1000000) };
    // AHB2: use AHB1 clock
    let ptr = (memory::SUNXI_CCU_BASE + 0x5c) as *mut u32;
    unsafe { write_volatile(ptr, 0) };
    // CPU: use OSC24M
    let ptr = (memory::SUNXI_CCU_BASE + 0x50) as *mut u32;
    unsafe { write_volatile(ptr, 0x10000) };

    // Turn off PLLs.
    for i in 0..6 {
        let ptr = (memory::SUNXI_CCU_BASE + i * 8) as *mut u32;
        bits::clrbits_32(ptr, 1 << 31);
    }
    match socid {
        SoCID::H5 => {
            let ptr = (memory::SUNXI_CCU_BASE + 0x44) as *mut u32;
            bits::clrbits_32(ptr, 1 << 31);
        }
        SoCID::A64 => {
            let ptr = (memory::SUNXI_CCU_BASE + 0x2c) as *mut u32;
            bits::clrbits_32(ptr, 1 << 31);
            let ptr = (memory::SUNXI_CCU_BASE + 0x4c) as *mut u32;
            bits::clrbits_32(ptr, 1 << 31);
        }
        _ => (),
    }
}

pub(crate) fn system_reset() {
    gic::v2::cpuif_disable();

    if cpu::scpi_available() {
        // Send the system reset request to the SCP
        match scpi::sys_power_state(scpi::ScpiSystemState::Reboot) {
            scpi::ScpiResult::Ok => (),
            ret => {
                uart::puts("PSCI: SCPI reboot failed: ");
                uart::decimal(ret as u64);
                uart::puts("\n");
            }
        }
    }

    unsafe {
        // Reset the whole system when the watchdog times out
        write_volatile(SUNXI_WDOG0_CFG_REG, 1);
        // Enable the watchdog with the shortest timeout (0.5 seconds)
        write_volatile(SUNXI_WDOG0_MODE_REG, 1);
    }
    // Wait for twice the watchdog timeout before panicking
    delays::wait_milisec(1000);

    uart::puts("PSCI: System reset failed\n");
    aarch64::cpu::wait_interrupt();
    panic!("failed to reset");
}
