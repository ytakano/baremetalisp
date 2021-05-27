pub(super) mod memory;

use super::BSPInit;

pub const SYSCNT_FRQ: u32 = 24000000;

pub(super) struct Init {}

impl BSPInit for Init {
    fn early_init() {}
    fn init() {}
}
