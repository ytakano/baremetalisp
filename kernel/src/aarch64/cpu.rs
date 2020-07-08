pub fn get_affinity_lv0() -> u64 {
    let mut ret: u64;

    unsafe {
        llvm_asm!(
            "mrs x0, mpidr_el1;
             and $0, x0, #0xFF"
            : "=r"(ret)
            :
            : "x0")
    }

    ret
}

pub fn send_event() {
    unsafe { llvm_asm!("sev"); }
}

pub fn wait_event() {
    unsafe { llvm_asm!("wfe"); }
}

pub fn start_non_primary() {
    if cfg!(feature = "raspi3") {
        unsafe {
            llvm_asm!(
                "mov x1, #0xe0
                 ldr x2, =_start
                 str x2, [x1]
                 str x2, [x1,  8] // core #2
                 str x2, [x1, 16] // core #3"
            );
        }
    }
}