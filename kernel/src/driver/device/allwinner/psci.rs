use super::cpu;
use crate::driver::arm::scpi;
use crate::driver::psci;
use crate::driver::psci::PsciResult;

pub(crate) const PLAT_MAX_PWR_LVL: usize = 2;

pub(crate) fn cpu_standby(cpu_state: u8) {}

pub(crate) fn pwr_domain_on(mpidr: usize) -> PsciResult {
    // validation
    match driver::topology::core_pos_by_mpidr(mpidr) {
        Some(_) => (),
        None => {
            return PsciResult::PsciEInternFail;
        }
    }

    if cpu::scpi_available() {
        scpi::scpi_set_css_power_state(
            mpidr,
            scpi::ScpiPowerState::ScpiPowerOn,
            scpi::ScpiPowerState::ScpiPowerOn,
            scpi::ScpiPowerState::ScpiPowerOn,
        );
    } else {
        cpu::cpu_on(mpidr);
    }

    PsciResult::PsciESuccess
}

pub(crate) fn pwr_domain_off(target_state: &psci::PsciPowerState) {}

pub(crate) fn pwr_domain_suspend_pwrdown_early(target_state: &psci::PsciPowerState) {}

pub(crate) fn pwr_domain_suspend(target_state: &psci::PsciPowerState) {}

pub(crate) fn pwr_domain_on_finish(target_state: &psci::PsciPowerState) {}

pub(crate) fn pwr_domain_on_finish_late(target_state: &psci::PsciPowerState) {}

pub(crate) fn pwr_domain_suspend_finish(target_state: &psci::PsciPowerState) {}

pub(crate) fn pwr_domain_pwr_down_wfi(target_state: &psci::PsciPowerState) {}

pub(crate) fn system_off() {}

pub(crate) fn system_reset() {}

pub(crate) fn validate_power_state(
    power_state: usize,
    req_state: &mut psci::PsciPowerState,
) -> isize {
    PsciResult::PsciENotSupported as isize
}

pub(crate) fn validate_ns_entrypoint(ns_entrypoint: usize) -> isize {
    PsciResult::PsciENotSupported as isize
}

pub(crate) fn get_sys_suspend_power_state(target_state: &psci::PsciPowerState) {}

pub(crate) fn get_pwr_lvl_state_idx(pwr_domain_state: u8, pwrlvl: isize) -> isize {
    PsciResult::PsciENotSupported as isize
}

pub(crate) fn translate_power_state_by_mpidr(
    mpidr: usize,
    power_state: usize,
    output_state: &mut psci::PsciPowerState,
) -> isize {
    PsciResult::PsciENotSupported as isize
}

pub(crate) fn get_node_hw_state(mpidr: usize, power_level: usize) -> isize {
    PsciResult::PsciENotSupported as isize
}

pub(crate) fn mem_protect_chk(base: usize, length: usize) -> isize {
    PsciResult::PsciENotSupported as isize
}

pub(crate) fn read_mem_protect(val: &mut isize) -> isize {
    PsciResult::PsciENotSupported as isize
}

pub(crate) fn write_mem_protect(val: isize) -> isize {
    PsciResult::PsciENotSupported as isize
}

pub(crate) fn system_reset2(is_vendor: isize, reset_type: isize, cookie: u64) -> isize {
    PsciResult::PsciENotSupported as isize
}
