use super::data;
use crate::driver::defs;
use crate::driver::topology;

/// Function which initializes the 'psci_non_cpu_pd_nodes' or the
/// 'psci_cpu_pd_nodes' corresponding to the power level.
pub fn init_pwr_domain_node(node_idx: usize, parent_idx: usize, level: u8) {
    if level > data::PSCI_CPU_PWR_LVL {
        data::set_non_cpu_pd_level(node_idx, level);
        data::set_non_cpu_pd_lock_index(node_idx, node_idx);
        data::set_non_cpu_pd_parent_node(node_idx, parent_idx);
        data::set_non_cpu_pd_local_state(node_idx, defs::MAX_OFF_STATE);
    } else {
        data::set_cpu_pd_parent_node(node_idx, parent_idx);

        // Initialize with an invalid mpidr
        data::set_cpu_pd_mpidr(node_idx, data::INVALID_MPIDR);

        // Set the Affinity Info for the cores as OFF
        data::set_cpu_aff_info_state(node_idx, data::AffInfoState::StateOff);

        // Invalidate the suspend level for the cpu
        data::set_cpu_target_pwrlvl(node_idx, data::INVALID_PWR_LVL);

        // Set the power state to OFF state
        data::set_cpu_local_state(node_idx, defs::MAX_OFF_STATE);
    }
}

/// Core routine to populate the power domain tree. The tree descriptor passed by
/// the platform is populated breadth-first and the first entry in the map
/// informs the number of root power domains. The parent nodes of the root nodes
/// will point to an invalid entry(-1).
pub(crate) fn populate_power_domain_tree(topology: &[u8]) -> u32 {
    let mut level = defs::MAX_PWR_LVL;
    let mut node_index = 0;
    let mut parent_node_index = 0;
    let mut num_nodes_at_lvl = 1;
    let mut n = 0;

    // For each level the inputs are:
    // - number of nodes at this level in plat_array i.e. num_nodes_at_level
    //   This is the sum of values of nodes at the parent level.
    // - Index of first entry at this level in the plat_array i.e.
    //   parent_node_index.
    // - Index of first free entry in psci_non_cpu_pd_nodes[] or
    //   psci_cpu_pd_nodes[] i.e. node_index depending upon the level.
    loop {
        let mut num_nodes_at_next_lvl = 0;

        // For each entry (parent node) at this level in the plat_array:
        // - Find the number of children
        // - Allocate a node in a power domain array for each child
        // - Set the parent of the child to the parent_node_index - 1
        // - Increment parent_node_index to point to the next parent
        // - Accumulate the number of children at next level.
        for _ in 0..num_nodes_at_lvl {
            let num_children = topology[parent_node_index] as u32;
            n = node_index;
            for j in node_index..(node_index + num_children) {
                init_pwr_domain_node(j as usize, parent_node_index - 1, level);
                n += 1;
            }

            node_index = n;
            num_nodes_at_next_lvl += num_children;
            parent_node_index += 1;
        }

        num_nodes_at_lvl = num_nodes_at_next_lvl;

        if level == data::PSCI_CPU_PWR_LVL {
            break;
        }

        level -= 1;

        // Reset the index for the cpu power domain array
        if level == data::PSCI_CPU_PWR_LVL {
            node_index = 0;
        }
    }

    n
}

/// PSCI helper function to get the parent nodes corresponding to a cpu_index.
fn get_parent_pwr_domain_nodes(cpu_idx: usize) -> [usize; defs::MAX_PWR_LVL as usize] {
    let mut parent_node = data::get_cpu_pd_parent_node(cpu_idx);
    let mut node_index = [0; defs::MAX_PWR_LVL as usize];

    for n in node_index.iter_mut() {
        *n = parent_node;
        parent_node = data::get_cpu_pd_parent_node(parent_node);
    }

    node_index
}

/// This functions updates cpu_start_idx and ncpus field for each of the node in
/// psci_non_cpu_pd_nodes[]. It does so by comparing the parent nodes of each of
/// the CPUs and check whether they match with the parent of the previous
/// CPU. The basic assumption for this work is that children of the same parent
/// are allocated adjacent indices. The platform should ensure this though proper
/// mapping of the CPUs to indices via plat_core_pos_by_mpidr() and
/// plat_my_core_pos() APIs.
pub(crate) fn update_pwrlvl_limits() {
    let mut nodes_idx = [0; defs::MAX_PWR_LVL as usize];
    for cpu_idx in 0..topology::CORE_COUNT {
        let parents = get_parent_pwr_domain_nodes(cpu_idx);
        for j in (0..defs::MAX_PWR_LVL as usize).rev() {
            if parents[j] != nodes_idx[j] {
                nodes_idx[j] = parents[j];
                data::set_non_cpu_pd_cpu_start_idx(nodes_idx[j], cpu_idx);
            }
            let ncpus = data::get_non_cpu_pd_ncpus(nodes_idx[j]);
            data::set_non_cpu_pd_ncpus(nodes_idx[j], ncpus);
        }
    }
}

/// This function initializes the psci_req_local_pwr_states.
pub(crate) fn init_req_local_pwr_states() {
    for pwrlvl in 1..(defs::MAX_PWR_LVL as usize + 1) {
        for core in 0..topology::CORE_COUNT {
            data::set_req_local_pwr_state(pwrlvl, core, defs::MAX_OFF_STATE);
        }
    }
}

/// This function is invoked post CPU power up and initialization. It sets the
/// affinity info state, target power state and requested power state for the
/// current CPU and all its ancestor power domains to RUN.
pub(crate) fn set_pwr_domains_to_run() {
    let cpu_idx = topology::core_pos();
    let mut parent_idx = data::get_cpu_pd_parent_node(cpu_idx);

    // Reset the local_state to RUN for the non cpu power domains.
    for lvl in (data::PSCI_CPU_PWR_LVL + 1)..(defs::MAX_PWR_LVL + 1) {
        data::set_non_cpu_pd_local_state(parent_idx, data::PSCI_LOCAL_STATE_RUN);
        data::set_req_local_pwr_state(lvl as usize, cpu_idx, data::PSCI_LOCAL_STATE_RUN);
        parent_idx = data::get_non_cpu_pd_parent_node(parent_idx);
    }

    // Set the affinity info state to ON
    data::set_cpu_aff_info_state(cpu_idx, data::AffInfoState::StateOn);

    data::set_cpu_local_state(cpu_idx, data::PSCI_LOCAL_STATE_RUN);
}
