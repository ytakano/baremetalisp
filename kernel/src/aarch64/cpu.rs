
pub fn get_affinity_lv0() -> u64 {
    let mut ret: u64;

    unsafe {
        asm!(
            "mrs x0, mpidr_el1;
             and $0, x0, #0xFF"
            : "=r"(ret)
            :
            : "x0")
    }

    ret
}