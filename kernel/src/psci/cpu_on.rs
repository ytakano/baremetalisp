use super::ep_info::EntryPointInfo;
use super::{psci_lock, AffInfoState, PsciResult, PSCI_CPU_DATA};
use crate::aarch64::cache;
use crate::driver;

use core::mem::size_of;
use core::ptr::read_volatile;

/// This function checks whether a cpu which has been requested to be turned on
/// is OFF to begin with.
fn cpu_on_validate_state(aff_state: &AffInfoState) -> PsciResult {
    match aff_state {
        AffInfoState::StateOn => PsciResult::PsciEAleadyOn,
        AffInfoState::StateOnPending => PsciResult::PsciEOnPending,
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
pub(crate) fn psci_cpu_on_start(target_cpu: usize, _ep: EntryPointInfo) -> PsciResult {
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
    psci_lock(idx);

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
    cache::clean_invalidate(
        unsafe { &mut PSCI_CPU_DATA[idx].aff_info_state },
        size_of::<AffInfoState>(),
    );
    let state = unsafe { read_volatile(&PSCI_CPU_DATA[idx].aff_info_state) };
    match cpu_on_validate_state(&state) {
        PsciResult::PsciESuccess => (),
        err => {
            return err;
        }
    }

    PsciResult::PsciENotSupported
}
