pub fn init() -> () {
    init_exceptions()
}

fn init_exceptions() -> () {
    unsafe {
        asm!(
            ".balign 0x800
             mov x0, xzr"
//            "vt_el3:"
//            "mov x0, xzr;"
//            "mov x0, xzr;"
            : : : :
        );
    };
    ()
}