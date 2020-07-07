/// Wait N CPU cycles (ARM CPU only)
pub fn wait_cycles(n: u32) {
    if n > 0 {
        for _ in 0..n {
            unsafe { llvm_asm!("nop;") };
        }
    }
}

/// Wait N microsec (ARM CPU only)
pub fn wait_microsec(n: u32) {
    // get the current counter frequency
    let mut frq: u64;
    unsafe { llvm_asm!("mrs %0, cntfrq_el0" : "=r"(frq)) };

    // read the current counter
    let mut t: u64;
    unsafe { llvm_asm!("mrs %0, cntpct_el0" : "=r"(t)) };

    t += ((frq / 1000) * n as u64) / 1000;

    let mut r: u64;
    unsafe { llvm_asm!("mrs %0, cntpct_el0" : "=r"(r)) };
    while r < t {
        unsafe { llvm_asm!("mrs %0, cntpct_el0" : "=r"(r)) };
    }
}

pub fn infinite_loop() -> ! {
    loop {
        unsafe { llvm_asm!("wfe") };
    }
}