use super::cpu;
use super::defs;
use super::memory;
use super::power;
use crate::driver::arm::{gic, scpi};
use crate::driver::psci::PsciResult;
use crate::driver::{psci, topology};
use crate::psci::is_local_state_off;

pub(crate) fn init() {
    cpu::init();
}

pub(crate) fn cpu_standby(_cpu_state: u8) {}

pub(crate) fn pwr_domain_on(mpidr: usize) -> PsciResult {
    // validation
    match topology::core_pos_by_mpidr(mpidr) {
        Some(_) => (),
        None => {
            return PsciResult::PsciEInternFail;
        }
    }

    if cpu::scpi_available() {
        scpi::set_css_power_state(
            mpidr,
            scpi::ScpiPowerState::PowerOn,
            scpi::ScpiPowerState::PowerOn,
            scpi::ScpiPowerState::PowerOn,
        );
    } else {
        cpu::cpu_on(mpidr);
    }

    PsciResult::PsciESuccess
}

pub(crate) fn pwr_domain_off(_target_state: &psci::PsciPowerState) {}

pub(crate) fn pwr_domain_suspend_pwrdown_early(_target_state: &psci::PsciPowerState) {}

pub(crate) fn pwr_domain_suspend(_target_state: &psci::PsciPowerState) {}

pub(crate) fn pwr_domain_on_finish(target_state: &psci::PsciPowerState) {
    if is_local_state_off(target_state[defs::SYSTEM_PWR_LVL as usize]) {
        gic::v2::distif_init();
    }

    if is_local_state_off(target_state[defs::CPU_PWR_LVL as usize]) {
        gic::v2::pcpu_distif_init();
        gic::v2::cpuif_enable();
    }
}

pub(crate) fn pwr_domain_on_finish_late(target_state: &psci::PsciPowerState) {
    if cpu::scpi_available() {
        pwr_domain_on_finish(target_state);
    }
}

pub(crate) fn pwr_domain_suspend_finish(_target_state: &psci::PsciPowerState) {}

pub(crate) fn pwr_domain_pwr_down_wfi(_target_state: &psci::PsciPowerState) {}

pub(crate) fn system_off() {
    power::system_off();
}

pub(crate) fn system_reset() {
    power::system_reset();
}

pub(crate) fn validate_power_state(
    _power_state: usize,
    _req_state: &mut psci::PsciPowerState,
) -> isize {
    PsciResult::PsciENotSupported as isize
}

pub(crate) fn validate_ns_entrypoint(ns_entrypoint: usize) -> PsciResult {
    if ns_entrypoint >= memory::DRAM_BASE as usize {
        PsciResult::PsciESuccess
    } else {
        PsciResult::PsciEInvalidAddress
    }
}

pub(crate) fn get_sys_suspend_power_state(_target_state: &psci::PsciPowerState) {}

pub(crate) fn get_pwr_lvl_state_idx(_pwr_domain_state: u8, _pwrlvl: isize) -> isize {
    PsciResult::PsciENotSupported as isize
}

pub(crate) fn translate_power_state_by_mpidr(
    _mpidr: usize,
    _power_state: usize,
    _output_state: &mut psci::PsciPowerState,
) -> isize {
    PsciResult::PsciENotSupported as isize
}

pub(crate) fn get_node_hw_state(_mpidr: usize, _power_level: usize) -> isize {
    PsciResult::PsciENotSupported as isize
}

pub(crate) fn mem_protect_chk(_base: usize, _length: usize) -> isize {
    PsciResult::PsciENotSupported as isize
}

pub(crate) fn read_mem_protect(_val: &mut isize) -> isize {
    PsciResult::PsciENotSupported as isize
}

pub(crate) fn write_mem_protect(_val: isize) -> isize {
    PsciResult::PsciENotSupported as isize
}

pub(crate) fn system_reset2(_is_vendor: isize, _reset_type: isize, _cookie: u64) -> isize {
    PsciResult::PsciENotSupported as isize
}
