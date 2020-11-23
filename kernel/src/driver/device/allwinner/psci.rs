use super::cpu;
use super::defs;
use super::memory;
use super::power;
use crate::aarch64;
use crate::driver::arm::{gic, scpi};
use crate::driver::{psci, topology};
use crate::psci::common::{is_local_state_off, is_local_state_retn, is_local_state_run};
use crate::psci::PsciResult;

pub(in crate::driver) fn init() {
    cpu::init();
}

pub fn available(f: u32) -> bool {
    use crate::psci as PS;
    match f {
        PS::PSCI_CPU_OFF
        | PS::PSCI_CPU_ON_AARCH32
        | PS::PSCI_CPU_ON_AARCH64
        | PS::PSCI_SYSTEM_OFF
        | PS::PSCI_SYSTEM_RESET => true,
        _ => false,
    }
}

pub(in crate::driver) fn cpu_standby(_cpu_state: u8) {}

pub(in crate::driver) fn pwr_domain_on(mpidr: usize) -> PsciResult {
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

fn scpi_map_state(psci_state: u8) -> scpi::ScpiPowerState {
    if is_local_state_run(psci_state) {
        scpi::ScpiPowerState::PowerOn
    } else if is_local_state_retn(psci_state) {
        scpi::ScpiPowerState::PowerRetention
    } else {
        scpi::ScpiPowerState::PowerOff
    }
}

pub(in crate::driver) fn pwr_domain_off(target_state: &psci::PsciPowerState) {
    let cpu_pwr_state = target_state[defs::CPU_PWR_LVL as usize];
    let cluster_pwr_state = target_state[defs::CLUSTER_PWR_LVL as usize];
    let system_pwr_state = target_state[defs::SYSTEM_PWR_LVL as usize];

    if is_local_state_off(cpu_pwr_state) {
        gic::v2::cpuif_disable();
    }

    let mpidr = aarch64::cpu::mpidr_el1::get() as usize;
    if cpu::scpi_available() {
        scpi::set_css_power_state(
            mpidr,
            scpi_map_state(cpu_pwr_state),
            scpi_map_state(cluster_pwr_state),
            scpi_map_state(system_pwr_state),
        );
    }
}

pub(in crate::driver) fn pwr_domain_suspend_pwrdown_early(_target_state: &psci::PsciPowerState) {}

pub(in crate::driver) fn pwr_domain_suspend(_target_state: &psci::PsciPowerState) {}

pub(in crate::driver) fn pwr_domain_on_finish(target_state: &psci::PsciPowerState) {
    if is_local_state_off(target_state[defs::SYSTEM_PWR_LVL as usize]) {
        gic::v2::distif_init();
    }

    if is_local_state_off(target_state[defs::CPU_PWR_LVL as usize]) {
        gic::v2::pcpu_distif_init();
        gic::v2::cpuif_enable();
    }
}

pub(in crate::driver) fn pwr_domain_on_finish_late(target_state: &psci::PsciPowerState) {
    if cpu::scpi_available() {
        pwr_domain_on_finish(target_state);
    }
}

pub(in crate::driver) fn pwr_domain_suspend_finish(_target_state: &psci::PsciPowerState) {}

pub(in crate::driver) fn pwr_domain_pwr_down_wfi(_target_state: &psci::PsciPowerState) -> bool {
    if cpu::scpi_available() {
        return false;
    }

    cpu::cpu_off(aarch64::cpu::mpidr_el1::get() as usize);

    loop {
        aarch64::cpu::wait_interrupt();
    }
}

pub(in crate::driver) fn system_off() {
    power::system_off();
}

pub(in crate::driver) fn system_reset() {
    power::system_reset();
}

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
