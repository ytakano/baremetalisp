pub mod uart;
pub mod memory;
pub mod mbox;

pub fn init() -> () {
    uart::init();
    init_exceptions()
}

fn init_exceptions() -> () {

    ()
}
