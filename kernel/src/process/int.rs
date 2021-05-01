use crate::aarch64::cpu;

pub(super) struct InterMask {
    prev: u64,
}

impl InterMask {
    pub(super) fn new() -> InterMask {
        // disable FIQ, IRQ, Abort, Debug
        let prev = cpu::daif::get();
        cpu::daif::set(prev | (cpu::DISABLE_ALL_EXCEPTIONS << cpu::SPSR_DAIF_SHIFT));

        InterMask { prev }
    }

    pub(super) fn unmask(self) {}
}

impl Drop for InterMask {
    fn drop(&mut self) {
        // restore DAIF
        cpu::daif::set(self.prev);
    }
}
