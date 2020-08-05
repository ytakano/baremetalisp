use crate::driver::psci;

pub(crate) const PLAT_MAX_PWR_LVL: usize = 2;

pub(crate) fn cpu_standby(cpu_state: u8) {}

pub(crate) fn pwr_domain_on(mpidr: u64) -> isize {
    psci::PSCI_E_NOT_SUPPORTED
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
    psci::PSCI_E_NOT_SUPPORTED
}

pub(crate) fn validate_ns_entrypoint(ns_entrypoint: usize) -> isize {
    psci::PSCI_E_NOT_SUPPORTED
}

pub(crate) fn get_sys_suspend_power_state(target_state: &psci::PsciPowerState) {}

pub(crate) fn get_pwr_lvl_state_idx(pwr_domain_state: u8, pwrlvl: isize) -> isize {
    psci::PSCI_E_NOT_SUPPORTED
}

pub(crate) fn translate_power_state_by_mpidr(
    mpidr: u64,
    power_state: usize,
    output_state: &mut psci::PsciPowerState,
) -> isize {
    psci::PSCI_E_NOT_SUPPORTED
}

pub(crate) fn get_node_hw_state(mpidr: u64, power_level: usize) -> isize {
    psci::PSCI_E_NOT_SUPPORTED
}

pub(crate) fn mem_protect_chk(base: u64, length: u64) -> isize {
    psci::PSCI_E_NOT_SUPPORTED
}

pub(crate) fn read_mem_protect(val: &mut isize) -> isize {
    psci::PSCI_E_NOT_SUPPORTED
}

pub(crate) fn write_mem_protect(val: isize) -> isize {
    psci::PSCI_E_NOT_SUPPORTED
}

pub(crate) fn system_reset2(is_vendor: isize, reset_type: isize, cookie: u64) -> isize {
    psci::PSCI_E_NOT_SUPPORTED
}
