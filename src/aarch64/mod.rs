pub mod uart;
pub mod memory;
pub mod mbox;
pub mod rand;
pub mod delays;
pub mod power;
pub mod graphics;

pub struct Context {
    pub graphics0: Option<graphics::Display>,
}

pub fn init() -> Context {
    uart::init();
    rand::init();
    let graphics0 = graphics::init();
    init_exceptions();

    Context{graphics0: graphics0}
}

fn init_exceptions() {

    ()
}
