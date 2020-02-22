pub mod uart;
pub mod memory;
pub mod mbox;
pub mod rand;
pub mod delays;

pub fn init() -> () {
    uart::init();
    rand::init();
    init_exceptions()
}

fn init_exceptions() -> () {

    ()
}
