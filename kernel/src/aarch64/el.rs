pub fn get_current_el() -> u32 {
    let mut el: u32;
    unsafe { asm!("mrs $0, CurrentEL" : "=r"(el)) }
    (el >> 2) & 0x3
}