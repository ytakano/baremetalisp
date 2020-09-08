use super::common;
use super::data;
use crate::aarch64::{cache, cpu};
use crate::driver;
use crate::driver::{defs, psci::PsciPowerState, topology};

/// Top level handler which is called when a cpu wants to power itself down.
/// It's assumed that along with turning the cpu power domain off, power
/// domains at higher levels will be turned off as far as possible. It finds
/// the highest level where a domain has to be powered off by traversing the
/// node information and then performs generic, architectural, platform setup
/// and state management required to turn OFF that power domain and domains
/// below it. e.g. For a cpu that's to be powered OFF, it could mean programming
/// the power controller whereas for a cluster that's to be powered off, it will
/// call the platform specific code which will disable coherency at the
/// interconnect level if the cpu is the last in the cluster and also the
/// program the power controller.
pub(crate) fn do_off(end_pwrlvl: usize) {
    let idx = topology::core_pos();
    data::flush_cache_cpu_state(idx);

    // Construct the psci_power_state for CPU_OFF
    let mut state_info = power_off_state();

    // Get the parent nodes here, this is important to do before we
    // initiate the power down sequence as after that point the core may
    // have exited coherency and its cache may be disabled, any access to
    // shared memory after that (such as the parent node lookup in
    // psci_cpu_pd_nodes) can cause coherency issues on some platforms.
    let parent_nodes = &common::get_parent_pwr_domain_nodes(idx)[0..end_pwrlvl];

    // This function acquires the lock corresponding to each power
    // level so that by the time all locks are taken, the system topology
    // is snapshot and state management can be done safely.
    for i in parent_nodes {
        unsafe { data::non_cpu_pd_force_lock(*i) };
    }

    // This function is passed the requested state info and
    // it returns the negotiated state info for each power level upto
    // the end level specified.
    common::do_state_coordination(end_pwrlvl, &mut state_info);

    for s in &state_info {
        uart::puts("state_info = ");
        uart::decimal(*s as u64);
        uart::puts("\n");
    }

    let max_off_lvl = common::find_max_off_lvl(&state_info);
    uart::puts("max_off_lvl = ");
    uart::decimal(max_off_lvl as u64);
    uart::puts("\n");

    // TODO
    // Arch. management. Initiate power down sequence.
    // psci_do_pwrdown_sequence(psci_find_max_off_lvl(&state_info));

    use crate::driver::uart;
    uart::puts("cpu_off\n");

    // Release the locks corresponding to each power level in the
    // reverse order to which they were acquired.
    for i in parent_nodes.iter().rev() {
        unsafe { data::non_cpu_pd_force_unlock(*i) };
    }

    // Plat. management: Perform platform specific actions to turn this
    // cpu off e.g. exit cpu coherency, program the power controller etc.
    driver::psci::pwr_domain_off(&state_info);
    uart::puts("cpu_off after\n");

    let ncpus = data::get_non_cpu_pd_ncpus(1);
    uart::puts("parent_idx = ");
    uart::decimal(1 as u64);
    uart::puts("\nncpus = ");
    uart::decimal(ncpus as u64);
    uart::puts("\n");

    loop {
        data::set_cpu_aff_info_state(idx, data::AffInfoState::StateOff);

        cache::flush_global();

        if !driver::psci::pwr_domain_pwr_down_wfi(&state_info) {
            cpu::dmb_sy();
            cpu::wait_interrupt();
            panic!("failed CPU off");
        }
    }
}

/// Construct the psci_power_state to request power OFF at all power levels.
fn power_off_state() -> PsciPowerState {
    [defs::MAX_OFF_STATE; defs::MAX_PWR_LVL as usize + 1]
}
