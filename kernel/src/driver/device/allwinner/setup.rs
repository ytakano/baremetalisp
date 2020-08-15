use super::psci;

pub fn platform_setup() {
    psci::init();
}
