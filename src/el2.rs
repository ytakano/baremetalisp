use crate::driver;

#[no_mangle]
pub fn el2_entry() -> ! {
    driver::uart::puts("entered EL2\n");

    loop{}
}