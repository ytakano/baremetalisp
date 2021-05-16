use crate::driver::setup;

pub(in crate::driver) struct Setup {}

impl setup::Setup for Setup {
    // TODO:
    // dummy
    fn early_platform_setup() {}
    fn platform_setup() {}
}
