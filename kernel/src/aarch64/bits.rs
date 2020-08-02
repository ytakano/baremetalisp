/// counting leading zero
pub fn clz(n: u64) -> u64 {
    let rd: u64;
    unsafe { asm!("clz {}, {}", lateout(reg) rd, in(reg) n) }
    rd
}

/// reverse bits
pub fn reverse(n: u64) -> u64 {
    let rd: u64;
    unsafe { asm!("rbit {}, {}", lateout(reg) rd, in(reg) n) }
    rd
}

/// counting tailing zero
pub fn tlz(n: u64) -> u64 {
    clz(reverse(n))
}
