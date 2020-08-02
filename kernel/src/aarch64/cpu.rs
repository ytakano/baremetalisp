pub fn get_affinity_lv0() -> u64 {
    let mpidr: u64;
    unsafe {
        asm!("mrs {}, mpidr_el1", lateout(reg) mpidr);
    }

    mpidr & 0xFF
}

pub fn get_affinity_lv1() -> u64 {
    let mpidr: u64;
    unsafe {
        asm!("mrs {}, mpidr_el1", lateout(reg) mpidr);
    }

    (mpidr >> 8) & 0xFF
}

pub fn get_current_el() -> u32 {
    let el: u64;
    unsafe { asm!("mrs {}, CurrentEL", lateout(reg) el) }
    ((el >> 2) & 0x3) as u32
}

pub fn send_event() {
    unsafe {
        asm!("sev");
    }
}

pub fn wait_event() {
    unsafe {
        asm!("wfe");
    }
}

pub fn start_non_primary() {
    if cfg!(feature = "raspi3") {
        unsafe {
            asm!(
                "mov x1, #0xe0
                 ldr x2, =_start
                 str x2, [x1]
                 str x2, [x1,  8] // core #2
                 str x2, [x1, 16] // core #3"
            );
        }
    }
}
