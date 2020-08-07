use super::{EntryPointInfo, PsciResult};

/// Generic handler which is called to physically power on a cpu identified by
/// its mpidr. It performs the generic, architectural, platform setup and state
/// management to power on the target cpu e.g. it will ensure that
/// enough information is stashed for it to resume execution in the non-secure
/// security state.
///
/// The state of all the relevant power domains are changed after calling the
/// platform handler as it can return error.
pub(crate) fn psci_cpu_on_start(target_cpu: usize, ep: &EntryPointInfo) -> PsciResult {
    PsciResult::PsciENotSupported
}
