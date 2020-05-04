/// counting leading zero
pub fn clz(n: u64) -> u64 {
    let rd: u64;
    unsafe { llvm_asm!("clz $0, $1": "=r"(rd) : "r"(n)) }
    rd
}

/// reverse bits
pub fn reverse(n: u64) -> u64 {
    let rd: u64;
    unsafe { llvm_asm!("rbit $0, $1": "=r"(rd) : "r"(n)) }
    rd
}

/// counting tailing zero
pub fn tlz(n: u64) -> u64 {
    clz(reverse(n))
}