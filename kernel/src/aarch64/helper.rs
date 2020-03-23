/// counting leading zero
pub fn clz(n: u64) -> u64 {
    let rd: u64;
    unsafe { asm!("clz $0, $1": "=r"(rd) : "r"(n)) }
    rd
}