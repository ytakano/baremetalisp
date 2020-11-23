use super::ep_info::{Aapcs64Params, EntryPointInfo, ParamHeader};
use super::PsciResult;
use super::{common, cpu_on, data, ep_info};
use crate::aarch64::{context, cpu};
use crate::driver;
use crate::driver::{defs, topology, uart};

use core::mem::size_of;

/// Function which initializes the 'psci_non_cpu_pd_nodes' or the
/// 'psci_cpu_pd_nodes' corresponding to the power level.
pub(super) fn init_pwr_domain_node(node_idx: usize, parent_idx: usize, level: u8) {
    if level > data::PSCI_CPU_PWR_LVL {
        data::set_non_cpu_pd_level(node_idx, level);
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
pub(super) fn populate_power_domain_tree(topology: &[u8]) -> u32 {
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

/// This functions updates cpu_start_idx and ncpus field for each of the node in
/// psci_non_cpu_pd_nodes[]. It does so by comparing the parent nodes of each of
/// the CPUs and check whether they match with the parent of the previous
/// CPU. The basic assumption for this work is that children of the same parent
/// are allocated adjacent indices. The platform should ensure this though proper
/// mapping of the CPUs to indices via plat_core_pos_by_mpidr() and
/// plat_my_core_pos() APIs.
pub(super) fn update_pwrlvl_limits() {
    let mut nodes_idx = [0; defs::MAX_PWR_LVL as usize];
    for cpu_idx in 0..topology::CORE_COUNT {
        let parents = common::get_parent_pwr_domain_nodes(cpu_idx);
        for j in (0..defs::MAX_PWR_LVL as usize).rev() {
            if parents[j] != nodes_idx[j] {
                nodes_idx[j] = parents[j];
                data::set_non_cpu_pd_cpu_start_idx(nodes_idx[j], cpu_idx);
            }
            let ncpus = data::get_non_cpu_pd_ncpus(nodes_idx[j]);
            data::set_non_cpu_pd_ncpus(nodes_idx[j], ncpus + 1);
        }
    }
}

/// This function initializes the psci_req_local_pwr_states.
pub(super) fn init_req_local_pwr_states() {
    for pwrlvl in 1..(defs::MAX_PWR_LVL as usize + 1) {
        for core in 0..topology::CORE_COUNT {
            data::set_req_local_pwr_state(pwrlvl, core, defs::MAX_OFF_STATE);
        }
    }
}

/// This function is invoked post CPU power up and initialization. It sets the
/// affinity info state, target power state and requested power state for the
/// current CPU and all its ancestor power domains to RUN.
fn set_pwr_domains_to_run(end_pwrlvl: u8) {
    let cpu_idx = topology::core_pos();
    let mut parent_idx = data::get_cpu_pd_parent_node(cpu_idx);

    // Reset the local_state to RUN for the non cpu power domains.
    for lvl in (data::PSCI_CPU_PWR_LVL + 1)..(end_pwrlvl + 1) {
        data::set_non_cpu_pd_local_state(parent_idx, data::PSCI_LOCAL_STATE_RUN);
        data::set_req_local_pwr_state(lvl as usize, cpu_idx, data::PSCI_LOCAL_STATE_RUN);
        parent_idx = data::get_non_cpu_pd_parent_node(parent_idx);
    }

    // Set the affinity info state to ON
    data::set_cpu_aff_info_state(cpu_idx, data::AffInfoState::StateOn);

    data::set_cpu_local_state(cpu_idx, data::PSCI_LOCAL_STATE_RUN);
}

/// Generic handler which is called when a cpu is physically powered on. It
/// traverses the node information and finds the highest power level powered
/// off and performs generic, architectural, platform setup and state management
/// to power on that power level and power levels below it.
/// e.g. For a cpu that's been powered on, it will call the platform specific
/// code to enable the gic cpu interface and for a cluster it will enable
/// coherency at the interconnect level in addition to gic cpu interface.
pub(super) fn init_warmboot() {
    let idx = topology::core_pos();
    match data::get_cpu_aff_info_state(idx) {
        data::AffInfoState::StateOff => {
            uart::puts("Unexpected affinity info state.\n");
            panic!("invalid affinity info");
        }
        _ => (),
    }

    // Get the maximum power domain level to traverse to after this cpu
    // has been physically powered up.
    let end_pwrlvl = get_power_on_target_pwrlvl(idx);

    // Get the parent nodes
    let parent_nodes = &common::get_parent_pwr_domain_nodes(idx)[0..end_pwrlvl as usize];

    // lock parents
    for i in parent_nodes {
        unsafe { data::non_cpu_pd_force_lock(*i) };
    }

    let state_info = common::get_target_local_pwr_states(end_pwrlvl);

    // This CPU could be resuming from suspend or it could have just been
    // turned on. To distinguish between these 2 cases, we examine the
    // affinity state of the CPU:
    //  - If the affinity state is ON_PENDING then it has just been
    //    turned on.
    //  - Else it is resuming from suspend.
    //
    // Depending on the type of warm reset identified, choose the right set
    // of power management handler and perform the generic, architecture
    // and platform specific handling.
    match data::get_cpu_aff_info_state(idx) {
        data::AffInfoState::StateOnPending => {
            cpu_on::finish(idx, &state_info);
        }
        _ => {
            // TODO
            // psci_cpu_suspend_finish(cpu_idx, &state_info);
        }
    }

    // Set the requested and target state of this CPU and all the higher
    // power domains which are ancestors of this CPU to run.
    set_pwr_domains_to_run(end_pwrlvl);

    // This loop releases the lock corresponding to each power level
    // in the reverse order to which they were acquired.
    for i in parent_nodes.iter().rev() {
        unsafe { data::non_cpu_pd_force_unlock(*i) };
    }
}

/// Routine to return the maximum power level to traverse to after a cpu has
/// been physically powered up. It is expected to be called immediately after
/// reset from assembler code.
fn get_power_on_target_pwrlvl(idx: usize) -> u8 {
    // Assume that this cpu was suspended and retrieve its target power
    // level. If it is invalid then it could only have been turned off
    // earlier. PLAT_MAX_PWR_LVL will be the highest power level a
    // cpu can be turned off to.
    let pwrlvl = data::get_cpu_target_pwrlvl(idx);
    if pwrlvl == data::INVALID_PWR_LVL {
        defs::MAX_PWR_LVL
    } else {
        pwrlvl
    }
}

extern "C" {
    fn ns_entry();
}

/// This function does the architectural setup and takes the warm boot
/// entry-point `mailbox_ep` as an argument. The function also initializes the
/// power domain topology tree by querying the platform. The power domain nodes
/// higher than the CPU are populated in the array psci_non_cpu_pd_nodes[] and
/// the CPU power domains are populated in psci_cpu_pd_nodes[]. The platform
/// exports its static topology map through the
/// populate_power_domain_topology_tree() API. The algorithm populates the
/// psci_non_cpu_pd_nodes and psci_cpu_pd_nodes iteratively by using this
/// topology map.  On a platform that implements two clusters of 2 cpus each,
/// and supporting 3 domain levels, the populated psci_non_cpu_pd_nodes would
/// look like this:
///
/// ---------------------------------------------------
/// | system node | cluster 0 node  | cluster 1 node  |
/// ---------------------------------------------------
///
/// And populated psci_cpu_pd_nodes would look like this :
/// <-    cpus cluster0   -><-   cpus cluster1   ->
/// ------------------------------------------------
/// |   CPU 0   |   CPU 1   |   CPU 2   |   CPU 3  |
/// ------------------------------------------------
pub(super) fn init() {
    // Populate the power domain arrays using the platform topology map
    populate_power_domain_tree(driver::topology::POWER_DOMAIN_TREE_DESC);

    // Update the CPU limits for each node in psci_non_cpu_pd_nodes */
    update_pwrlvl_limits();

    // Populate the mpidr field of cpu node for this CPU
    data::set_cpu_pd_mpidr(
        topology::core_pos(),
        cpu::mpidr_el1::get() & cpu::MPIDR_AFFINITY_MASK,
    );

    init_req_local_pwr_states();

    // Set the requested and target state of this CPU and all the higher
    // power domain levels for this CPU to run.
    set_pwr_domains_to_run(defs::MAX_PWR_LVL);

    // setup normal world's context
    let ep;
    let ptr = ns_entry as *const () as usize;
    match validate_entry_point(ptr, 0) {
        Ok(e) => {
            ep = e;
        }
        Err(_) => {
            return;
        }
    }

    // Store the re-entry information for the non-secure world.
    context::init_context(topology::core_pos(), ep);
}

/// This function validates the entrypoint with the platform layer if the
/// appropriate pm_ops hook is exported by the platform and returns the
/// 'entry_point_info'.
pub(super) fn validate_entry_point(
    entrypoint: usize,
    context_id: usize,
) -> Result<EntryPointInfo, PsciResult> {
    // Validate the entrypoint using platform psci_ops
    match driver::psci::validate_ns_entrypoint(entrypoint) {
        PsciResult::PsciESuccess => (),
        _ => {
            return Err(PsciResult::PsciEInvalidAddress);
        }
    }

    // Verify and derive the re-entry information for
    // the non-secure world from the non-secure state from
    // where this call originated.
    get_ns_ep_info(entrypoint, context_id)
}

/// This function determines the full entrypoint information for the requested
/// PSCI entrypoint on power on/resume and returns it.
/// (for AArch64)
fn get_ns_ep_info(entrypoint: usize, context_id: usize) -> Result<EntryPointInfo, PsciResult> {
    let ns_scr_el3 = cpu::scr_el3::get();
    let sctlr = if (ns_scr_el3 & cpu::SCR_HCE_BIT) != 0 {
        cpu::sctlr_el2::get()
    } else {
        cpu::sctlr_el1::get()
    };

    let ee;
    let ep_attr;
    if (sctlr & cpu::SCTLR_EE_BIT) != 0 {
        ep_attr = ep_info::EP_NON_SECURE | ep_info::EP_EE_BIG | ep_info::EP_ST_DISABLE;
        ee = 1;
    } else {
        ep_attr = ep_info::EP_NON_SECURE | ep_info::EP_ST_DISABLE;
        ee = 0;
    }

    // Figure out whether the cpu enters the non-secure address space
    // in aarch32 or aarch64
    let spsr = if (ns_scr_el3 & cpu::SCR_RW_BIT) != 0 {
        // Check whether a Thumb entry point has been provided for an
        // aarch64 EL
        if (entrypoint & 0x1) != 0 {
            return Err(PsciResult::PsciEInvalidAddress);
        }

        let mode = if (ns_scr_el3 & cpu::SCR_HCE_BIT) != 0 {
            cpu::EL::EL2h
        } else {
            cpu::EL::EL1h
        };

        cpu::spsr64(mode, cpu::DISABLE_ALL_EXCEPTIONS)
    } else {
        let mode = if (ns_scr_el3 & cpu::SCR_HCE_BIT) != 0 {
            cpu::MODE32_HYP
        } else {
            cpu::MODE32_SVC
        };

        // TODO: Choose async. exception bits if HYP mode is not
        // implemented according to the values of SCR.{AW, FW} bits
        let daif = cpu::DAIF_ABT_BIT | cpu::DAIF_IRQ_BIT | cpu::DAIF_FIQ_BIT;

        cpu::spsr32(mode, entrypoint as u64 & 1, ee, daif)
    };

    let headr = ParamHeader {
        htype: ep_info::PARAM_EP,
        version: ep_info::PARAM_VERSION_1,
        size: size_of::<ParamHeader>() as u16,
        attr: ep_attr as u32,
    };

    let mut args = Aapcs64Params::new();
    args.arg0 = context_id as u64;

    let ep = EntryPointInfo {
        h: headr,
        pc: entrypoint,
        spsr: spsr,
        args: args,
    };

    Ok(ep)
}
