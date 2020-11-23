use super::data;
use crate::driver;
use crate::driver::{defs, psci::PsciPowerState, topology};

/// PSCI helper function to get the parent nodes corresponding to a cpu_index.
pub(super) fn get_parent_pwr_domain_nodes(cpu_idx: usize) -> [usize; defs::MAX_PWR_LVL as usize] {
    let mut parent_node = data::get_cpu_pd_parent_node(cpu_idx);
    let mut node_index = [0; defs::MAX_PWR_LVL as usize];

    for n in node_index.iter_mut() {
        *n = parent_node;
        parent_node = data::get_non_cpu_pd_parent_node(parent_node);
    }

    node_index
}

/// Helper function to return the current local power state of each power domain
/// from the current cpu power domain to its ancestor at the 'end_pwrlvl'. This
/// function will be called after a cpu is powered on to find the local state
/// each power domain has emerged from.
pub(super) fn get_target_local_pwr_states(end_pwrlvl: u8) -> PsciPowerState {
    let mut target_state = [0; (defs::MAX_PWR_LVL + 1) as usize];
    let idx = topology::core_pos();
    let mut parent_idx = data::get_cpu_pd_parent_node(idx);

    // Copy the local power state from node to state_info
    target_state[data::PSCI_CPU_PWR_LVL as usize] = data::get_cpu_local_state(idx);
    for lvl in (data::PSCI_CPU_PWR_LVL + 1)..(end_pwrlvl + 1) {
        target_state[lvl as usize] = data::get_non_cpu_pd_local_state(parent_idx);
        parent_idx = data::get_non_cpu_pd_parent_node(parent_idx);
    }

    // Set the the higher levels to RUN
    for lvl in (end_pwrlvl + 1)..(defs::MAX_PWR_LVL + 1) {
        target_state[lvl as usize] = data::PSCI_LOCAL_STATE_RUN;
    }

    target_state
}

/// Helper function to set the target local power state that each power domain
/// from the current cpu power domain to its ancestor at the 'end_pwrlvl' will
/// enter. This function will be called after coordination of requested power
/// states has been done for each power level.
pub(super) fn set_target_local_pwr_states(end_pwlvl: u8, target_state: &PsciPowerState) {
    let idx = topology::core_pos();
    data::set_cpu_local_state(idx, target_state[data::PSCI_CPU_PWR_LVL as usize]);

    let mut parent_idx = data::get_cpu_pd_parent_node(idx);

    // Copy the local_state from state_info
    for lvl in 1..(end_pwlvl + 1) {
        data::set_non_cpu_pd_local_state(parent_idx, target_state[lvl as usize]);
        parent_idx = data::get_non_cpu_pd_parent_node(parent_idx);
    }
}

/// This function is passed the local power states requested for each power
/// domain (state_info) between the current CPU domain and its ancestors until
/// the target power level (end_pwrlvl). It updates the array of requested power
/// states with this information.
///
/// Then, for each level (apart from the CPU level) until the 'end_pwrlvl', it
/// retrieves the states requested by all the cpus of which the power domain at
/// that level is an ancestor. It passes this information to the platform to
/// coordinate and return the target power state. If the target state for a level
/// is RUN then subsequent levels are not considered. At the CPU level, state
/// coordination is not required. Hence, the requested and the target states are
/// the same.
///
/// The 'state_info' is updated with the target state for each level between the
/// CPU and the 'end_pwrlvl' and returned to the caller.
///
/// This function will only be invoked with data cache enabled and while
/// powering down a core.
pub(super) fn do_state_coordination(end_pwrlvl: usize, state_info: &mut PsciPowerState) {
    let cpu_idx = topology::core_pos();
    let mut parent_idx = data::get_cpu_pd_parent_node(cpu_idx);

    let mut n = data::PSCI_CPU_PWR_LVL as usize + 1;
    for lvl in (data::PSCI_CPU_PWR_LVL as usize + 1)..(end_pwrlvl + 1) {
        // First update the requested power state
        data::set_req_local_pwr_state(lvl, cpu_idx, state_info[lvl]);

        // Get the requested power states for this power level
        let start_idx = data::get_non_cpu_pd_cpu_start_idx(parent_idx);
        let req_states = data::get_req_local_pwr_states(lvl, start_idx);

        // Let the platform coordinate amongst the requested states at
        // this power level and return the target local power state.

        let ncpus = data::get_non_cpu_pd_ncpus(parent_idx);
        let target_state = driver::psci::get_target_pwr_state(lvl, req_states, ncpus);

        state_info[lvl] = target_state;

        n = lvl;

        // Break early if the negotiated target power state is RUN
        if is_local_state_run(target_state) {
            break;
        }

        parent_idx = data::get_non_cpu_pd_parent_node(parent_idx);
    }

    // This is for cases when we break out of the above loop early because
    // the target power state is RUN at a power level < end_pwrlvl.
    // We update the requested power state from state_info and then
    // set the target state as RUN.
    for lvl in (n + 1)..(end_pwrlvl + 1) {
        data::set_req_local_pwr_state(lvl, cpu_idx, state_info[lvl]);
        state_info[lvl] = data::PSCI_LOCAL_STATE_RUN;
    }

    // Update the target state in the power domain nodes
    set_target_local_pwr_states(end_pwrlvl as u8, state_info);
}

/// Function to test whether the plat_local_state is RUN state
pub fn is_local_state_run(plat_local_state: u8) -> bool {
    plat_local_state == data::PSCI_LOCAL_STATE_RUN
}

/// Function to test whether the plat_local_state is OFF state
pub fn is_local_state_off(plat_local_state: u8) -> bool {
    (plat_local_state > driver::defs::MAX_RET_STATE)
        && (plat_local_state <= driver::defs::MAX_OFF_STATE)
}

/// Function to test whether the plat_local_state is RETENTION state
pub fn is_local_state_retn(plat_local_state: u8) -> bool {
    (plat_local_state > data::PSCI_LOCAL_STATE_RUN)
        && (plat_local_state <= driver::defs::MAX_RET_STATE)
}

// This function finds the highest power level which will be powered down
// amongst all the power levels specified in the 'state_info' structure
pub fn find_max_off_lvl(state_info: &PsciPowerState) -> u8 {
    let mut i = driver::defs::MAX_PWR_LVL;
    loop {
        if is_local_state_off(state_info[i as usize]) {
            return i;
        }
        if i == data::PSCI_CPU_PWR_LVL {
            break;
        }
        i -= 1;
    }

    data::INVALID_PWR_LVL
}
