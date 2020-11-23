use super::memory;
use crate::driver::psci;
use crate::psci::PsciResult;

// TODO:
// not yet implemented

pub(in crate::driver) fn init() {}

pub(in crate::driver) fn cpu_standby(_cpu_state: u8) {}

pub(in crate::driver) fn pwr_domain_on(_mpidr: usize) -> PsciResult {
    PsciResult::PsciENotSupported
}

pub(in crate::driver) fn pwr_domain_off(_target_state: &psci::PsciPowerState) {}

pub(in crate::driver) fn pwr_domain_suspend_pwrdown_early(_target_state: &psci::PsciPowerState) {}

pub(in crate::driver) fn pwr_domain_suspend(_target_state: &psci::PsciPowerState) {}

pub(in crate::driver) fn pwr_domain_on_finish(_target_state: &psci::PsciPowerState) {}

pub(in crate::driver) fn pwr_domain_on_finish_late(_target_state: &psci::PsciPowerState) {}

pub(in crate::driver) fn pwr_domain_suspend_finish(_target_state: &psci::PsciPowerState) {}

pub(in crate::driver) fn pwr_domain_pwr_down_wfi(_target_state: &psci::PsciPowerState) -> bool {
    false
}

pub(in crate::driver) fn system_off() {}

pub(in crate::driver) fn system_reset() {}

pub(in crate::driver) fn validate_power_state(
    _power_state: usize,
    _req_state: &mut psci::PsciPowerState,
) -> isize {
    PsciResult::PsciENotSupported as isize
}

pub(in crate::driver) fn validate_ns_entrypoint(ns_entrypoint: usize) -> PsciResult {
    if ns_entrypoint >= memory::DRAM_BASE as usize {
        PsciResult::PsciESuccess
    } else {
        PsciResult::PsciEInvalidAddress
    }
}

pub(in crate::driver) fn get_sys_suspend_power_state(_target_state: &psci::PsciPowerState) {}

pub(in crate::driver) fn get_pwr_lvl_state_idx(_pwr_domain_state: u8, _pwrlvl: isize) -> isize {
    PsciResult::PsciENotSupported as isize
}

pub(in crate::driver) fn translate_power_state_by_mpidr(
    _mpidr: usize,
    _power_state: usize,
    _output_state: &mut psci::PsciPowerState,
) -> isize {
    PsciResult::PsciENotSupported as isize
}

pub(in crate::driver) fn get_node_hw_state(_mpidr: usize, _power_level: usize) -> isize {
    PsciResult::PsciENotSupported as isize
}

pub(in crate::driver) fn mem_protect_chk(_base: usize, _length: usize) -> isize {
    PsciResult::PsciENotSupported as isize
}

pub(in crate::driver) fn read_mem_protect(_val: &mut isize) -> isize {
    PsciResult::PsciENotSupported as isize
}

pub(in crate::driver) fn write_mem_protect(_val: isize) -> isize {
    PsciResult::PsciENotSupported as isize
}

pub(in crate::driver) fn system_reset2(
    _is_vendor: isize,
    _reset_type: isize,
    _cookie: u64,
) -> isize {
    PsciResult::PsciENotSupported as isize
}
