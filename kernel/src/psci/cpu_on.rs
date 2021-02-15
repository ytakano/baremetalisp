use super::data;
use super::data::AffInfoState;
use super::ep_info::EntryPointInfo;
use super::PsciResult;
use crate::aarch64;
use crate::driver;
use crate::driver::psci::PsciPowerState;
use crate::driver::uart;

/// This function checks whether a cpu which has been requested to be turned on
/// is OFF to begin with.
fn validate_state(aff_state: &AffInfoState) -> PsciResult {
    match aff_state {
        AffInfoState::StateOn => {
            uart::puts("PSCI: target CPU is already on\n");
            PsciResult::PsciEAleadyOn
        }
        AffInfoState::StateOnPending => {
            uart::puts("PSCI: target CPU is on pending on\n");
            PsciResult::PsciEOnPending
        }
        AffInfoState::StateOff => PsciResult::PsciESuccess,
    }
}

/// Generic handler which is called to physically power on a cpu identified by
/// its mpidr. It performs the generic, architectural, platform setup and state
/// management to power on the target cpu e.g. it will ensure that
/// enough information is stashed for it to resume execution in the non-secure
/// security state.
///
/// The state of all the relevant power domains are changed after calling the
/// platform handler as it can return error.
pub(crate) fn start(target_cpu: usize, ep: EntryPointInfo) -> PsciResult {
    let idx;
    match driver::topology::core_pos_by_mpidr(target_cpu) {
        Some(c) => {
            idx = c;
        }
        None => {
            return PsciResult::PsciEInvalidParams;
        }
    }

    // Protect against multiple CPUs trying to turn ON the same target CPU
    let _lock = data::cpu_lock(idx);

    // Generic management: Ensure that the cpu is off to be
    // turned on.
    // Perform cache maintanence ahead of reading the target CPU state to
    // ensure that the data is not stale.
    // There is a theoretical edge case where the cache may contain stale
    // data for the target CPU data - this can occur under the following
    // conditions:
    // - the target CPU is in another cluster from the current
    // - the target CPU was the last CPU to shutdown on its cluster
    // - the cluster was removed from coherency as part of the CPU shutdown
    //
    // In this case the cache maintenace that was performed as part of the
    // target CPUs shutdown was not seen by the current CPU's cluster. And
    // so the cache may contain stale data for the target CPU.
    data::flush_cache_cpu_state(idx);
    let state = data::get_cpu_aff_info_state(idx);
    match validate_state(&state) {
        PsciResult::PsciESuccess => (),
        err => {
            return err;
        }
    }

    // Set the Affinity info state of the target cpu to ON_PENDING.
    // Flush aff_info_state as it will be accessed with caches
    // turned OFF.
    data::set_cpu_aff_info_state(idx, AffInfoState::StateOnPending);
    data::flush_cache_cpu_state(idx);

    // The cache line invalidation by the target CPU after setting the
    // state to OFF (see psci_do_cpu_off()), could cause the update to
    // aff_info_state to be invalidated. Retry the update if the target
    // CPU aff_info_state is not ON_PENDING.
    match data::get_cpu_aff_info_state(idx) {
        AffInfoState::StateOnPending => (),
        _ => {
            data::set_cpu_aff_info_state(idx, AffInfoState::StateOnPending);
            data::flush_cache_cpu_state(idx);
        }
    }

    // Store the re-entry information for the non-secure world.
    aarch64::context::init_context(idx, ep);

    // Perform generic, architecture and platform specific handling.
    // Plat. management: Give the platform the current state
    // of the target cpu to allow it to perform the necessary
    // steps to power on.
    let rc = driver::psci::pwr_domain_on(target_cpu);
    match &rc {
        PsciResult::PsciESuccess => (),
        _ => {
            // Restore the state on error.
            data::set_cpu_aff_info_state(idx, AffInfoState::StateOff);
            data::flush_cache_cpu_state(idx);
        }
    }

    rc
}

pub(super) fn finish(idx: usize, state_info: &PsciPowerState) {
    // Plat. management: Perform the platform specific actions
    // for this cpu e.g. enabling the gic or zeroing the mailbox
    // register. The actual state of this cpu has already been
    // changed.
    driver::psci::pwr_domain_on_finish(state_info);

    // Plat. management: Perform any platform specific actions which
    // can only be done with the cpu and the cluster guaranteed to
    // be coherent.
    driver::psci::pwr_domain_on_finish_late(state_info);

    // Lock the CPU spin lock to make sure that the context initialization
    // is done. Since the lock is only used in this function to create
    // a synchronization point with cpu_on_start(), it can be released
    // immediately.
    {
        data::cpu_lock(idx);
    }

    // Populate the mpidr field within the cpu node array */
    // This needs to be done only once */
    data::set_cpu_pd_mpidr(
        idx,
        aarch64::cpu::mpidr_el1::get() & aarch64::cpu::MPIDR_AFFINITY_MASK,
    );
}
