use super::data;
use crate::aarch64::cpu;
use crate::driver;
use crate::driver::psci::PsciPowerState;

/// The following functions finish an earlier suspend request. They
/// are called by the common finisher routine in psci_common.c. The `state_info`
/// is the psci_power_state from which this CPU has woken up from.
pub(crate) fn finish(idx: usize, state_info: &PsciPowerState) {
    // Plat. management: Perform the platform specific actions
    // before we change the state of the cpu e.g. enabling the
    // gic or zeroing the mailbox register. If anything goes
    // wrong then assert as there is no way to recover from this
    // situation.
    driver::psci::pwr_domain_suspend_finish(state_info);

    // Re-init the cntfrq_el0 register
    cpu::cntfrq_el0::set(driver::defs::SYSCNT_FRQ as u64);

    // Invalidate the suspend level for the cpu
    data::set_cpu_target_pwrlvl(idx, data::INVALID_PWR_LVL);
}
