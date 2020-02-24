extern {
    fn mbox_get_serial() -> u64;
}

pub fn get_serial() -> u64 {
    unsafe { mbox_get_serial() }
}