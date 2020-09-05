#[cfg(feature = "pine64")]
use super::device::allwinner::psci;

#[cfg(any(feature = "raspi3", feature = "raspi4"))]
use super::device::raspi::psci;

use super::defs;

//use super::uart;

pub enum PsciResult {
    PsciESuccess = 0,
    PsciENotSupported = -1,
    PsciEInvalidParams = -2,
    PsciEDenied = -3,
    PsciEAleadyOn = -4,
    PsciEOnPending = -5,
    PsciEInternFail = -6,
    PsciENotPresent = -7,
    PsciEDisabled = -8,
    PsciEInvalidAddress = -9,
}

// The pwr_domain_state[] stores the local power state at each level
// for the CPU.
pub type PsciPowerState = [u8; (defs::MAX_PWR_LVL + 1) as usize];

pub fn cpu_standby(cpu_state: u8) {
    psci::cpu_standby(cpu_state);
}

pub fn pwr_domain_on(mpidr: usize) -> PsciResult {
    psci::pwr_domain_on(mpidr)
}

pub fn pwr_domain_off(target_state: &PsciPowerState) {
    psci::pwr_domain_off(target_state)
}

pub fn pwr_domain_suspend_pwrdown_early(target_state: &PsciPowerState) {
    psci::pwr_domain_suspend_pwrdown_early(target_state)
}

pub fn pwr_domain_suspend(target_state: &PsciPowerState) {
    psci::pwr_domain_suspend(target_state)
}

pub fn pwr_domain_on_finish(target_state: &PsciPowerState) {
    psci::pwr_domain_on_finish(target_state)
}

pub fn pwr_domain_on_finish_late(target_state: &PsciPowerState) {
    psci::pwr_domain_on_finish_late(target_state)
}

pub fn pwr_domain_suspend_finish(target_state: &PsciPowerState) {
    psci::pwr_domain_suspend_finish(target_state)
}

pub fn pwr_domain_pwr_down_wfi(target_state: &PsciPowerState) {
    psci::pwr_domain_pwr_down_wfi(target_state)
}

pub fn system_off() {
    psci::system_off()
}

pub fn system_reset() {
    psci::system_reset()
}

pub fn validate_power_state(power_state: usize, req_state: &mut PsciPowerState) -> isize {
    psci::validate_power_state(power_state, req_state)
}

pub fn validate_ns_entrypoint(ns_entrypoint: usize) -> PsciResult {
    psci::validate_ns_entrypoint(ns_entrypoint)
}

pub fn get_sys_suspend_power_state(target_state: &PsciPowerState) {
    psci::get_sys_suspend_power_state(target_state)
}

pub fn get_pwr_lvl_state_idx(pwr_domain_state: u8, pwrlvl: isize) -> isize {
    psci::get_pwr_lvl_state_idx(pwr_domain_state, pwrlvl)
}

pub fn translate_power_state_by_mpidr(
    mpidr: usize,
    power_state: usize,
    output_state: &mut PsciPowerState,
) -> isize {
    psci::translate_power_state_by_mpidr(mpidr, power_state, output_state)
}

pub fn get_node_hw_state(mpidr: usize, power_level: usize) -> isize {
    psci::get_node_hw_state(mpidr, power_level)
}

pub fn mem_protect_chk(base: usize, length: usize) -> isize {
    psci::mem_protect_chk(base, length)
}

pub fn read_mem_protect(val: &mut isize) -> isize {
    psci::read_mem_protect(val)
}

pub fn write_mem_protect(val: isize) -> isize {
    psci::write_mem_protect(val)
}

pub fn system_reset2(is_vendor: isize, reset_type: isize, cookie: u64) -> isize {
    psci::system_reset2(is_vendor, reset_type, cookie)
}
