use super::data;
use crate::driver::defs;

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
