use crate::driver;

#[no_mangle]
pub fn el1_entry() -> ! {
//    driver::uart::puts("entered EL1\n");

    loop{}
}