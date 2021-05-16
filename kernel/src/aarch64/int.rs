use super::cpu;
use crate::int::InterMask;

pub struct AA64Mask {
    prev: u64,
}

impl InterMask for AA64Mask {
    fn new() -> AA64Mask {
        // disable FIQ, IRQ, Abort, Debug
        let prev = cpu::daif::get();
        cpu::daif::set(prev | (cpu::DISABLE_ALL_EXCEPTIONS << cpu::SPSR_DAIF_SHIFT));

        AA64Mask { prev }
    }

    fn enable_irq() {
        let daif = cpu::daif::get();
        cpu::daif::set(daif & !((cpu::DAIF_IRQ_BIT | cpu::DAIF_FIQ_BIT) << cpu::SPSR_DAIF_SHIFT));
    }

    fn unmask(self) {}
}

impl Drop for AA64Mask {
    fn drop(&mut self) {
        // restore DAIF
        cpu::daif::set(self.prev);
    }
}
