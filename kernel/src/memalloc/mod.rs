mod buddy;

use crate::aarch64::mmu::PAGESIZE;
use crate::driver::uart;

pub fn test() {
    let mut allc = buddy::BuddyAlloc::new(PAGESIZE as usize, 0);

    let addr1 = match allc.mem_alloc(PAGESIZE as usize * 2) {
        Some(addr) => addr,
        None => {
            uart::puts("failed alloc addr1\n");
            return;
        }
    };

    let addr2 = match allc.mem_alloc(PAGESIZE as usize * 3) {
        Some(addr) => addr,
        None => {
            uart::puts("failed alloc addr2\n");
            return;
        }
    };

    let addr3 = match allc.mem_alloc(PAGESIZE as usize * 8) {
        Some(addr) => addr,
        None => {
            uart::puts("failed alloc addr3\n");
            return;
        }
    };

    allc.print();
    uart::puts("\n");

    allc.mem_free(addr2);
    allc.print();
    uart::puts("\n");

    allc.mem_free(addr1);
    allc.print();
    uart::puts("\n");

    allc.mem_free(addr3);
    allc.print();
}
