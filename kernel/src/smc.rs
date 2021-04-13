#[repr(C)]
pub struct ThreadSmcArgs {
    a0: u64,
    a1: u64,
    a2: u64,
    a3: u64,
    a4: u64,
    a5: u64,
    a6: u64,
    a7: u64,
}

#[no_mangle]
pub fn thread_handle_std_smc(
    _a0: u32,
    _a1: u32,
    _a2: u32,
    _a3: u32,
    _a4: u32,
    _a5: u32,
    _a6: u32,
    _a7: u32,
) -> u32 {
    0
}

#[no_mangle]
pub fn thread_handle_fast_smc(_args: &mut ThreadSmcArgs) {}

#[no_mangle]
pub fn thread_cpu_off_handler(_a0: u32, _a1: u32) -> u32 {
    0
}

#[no_mangle]
pub fn thread_cpu_resume_handler(_a0: u32, _a1: u32) -> u32 {
    0
}

#[no_mangle]
pub fn thread_cpu_suspend_handler(_a0: u32, _a1: u32) -> u32 {
    0
}

#[no_mangle]
pub fn itr_core_handler() {
    todo! {}
}

#[no_mangle]
pub fn thread_system_off_handler(_a0: u32, _a1: u32) -> u32 {
    0
}

#[no_mangle]
pub fn thread_system_reset_handler(_a0: u32, _a1: u32) -> u32 {
    0
}
