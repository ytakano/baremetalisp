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

pub fn dmb_st() {
    unsafe {
        asm!("dmb st");
    }
}

pub fn dmb_ld() {
    unsafe {
        asm!("dmb ld");
    }
}

pub fn dmb() {
    unsafe {
        asm!("dmb");
    }
}

pub fn start_non_primary() {
    if cfg!(feature = "raspi3") {
        unsafe {
            asm!(
                "mov {0}, #0xe0
                 ldr {1}, =_start
                 str {1}, [{0}]
                 str {1}, [{0},  8] // core #2
                 str {1}, [{0}, 16] // core #3",
            lateout(reg) _,
            lateout(reg) _
            );
        }
    }
}
