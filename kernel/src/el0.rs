use crate::driver;

#[no_mangle]
pub fn el0_entry() -> ! {
//    driver::uart::puts("entered EL0\n");

    loop{}
}