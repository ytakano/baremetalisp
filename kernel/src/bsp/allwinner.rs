pub(super) mod memory;

use super::BSPInit;

pub(super) struct Init {}

impl BSPInit for Init {
    fn early_init() {}
    fn init() {}
}
