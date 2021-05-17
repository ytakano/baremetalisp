pub(in crate::driver) type IRQManager = int_rpi::IRQManager;
pub(in crate::driver) type IRQNumber = int_rpi::IRQNumber;

#[cfg(feature = "raspi4")]
use crate::driver::gic as int_rpi;

#[cfg(feature = "raspi3")]
mod int_rpi {
    use crate::int::{self, IRQ};
    use arr_macro::arr;

    const MAX_LOCAL_IRQ_NUMBER: usize = 11;
    const MAX_PERIPHERAL_IRQ_NUMBER: usize = 63;

    pub enum IRQNumber {
        Private(u8),
        Peripheral(u8),
    }

    pub struct IRQManager {
        hdls_private: [Option<IRQ<IRQNumber>>; MAX_LOCAL_IRQ_NUMBER],
        hdls_periheral: [Option<IRQ<IRQNumber>>; MAX_PERIPHERAL_IRQ_NUMBER],
    }

    impl IRQManager {
        pub(in crate::driver) const fn new_mng() -> Self {
            IRQManager {
                hdls_private: arr![None; 11],
                hdls_periheral: arr![None; 63],
            }
        }
    }

    impl int::IRQManager for IRQManager {
        type IRQNumberType = IRQNumber;

        fn enable(&self, _irq_num: Self::IRQNumberType) {}
        fn disable(&self, _irq_num: Self::IRQNumberType) {}
        fn ack(&self, _irq_num: Self::IRQNumberType) {}
        fn handle(&self, _irq_num: Self::IRQNumberType) {}

        fn register_handler(
            &mut self,
            irq_num: Self::IRQNumberType,
            handler: IRQ<Self::IRQNumberType>,
        ) {
            match irq_num {
                IRQNumber::Private(n) => {
                    self.hdls_private[n as usize] = Some(handler);
                }
                IRQNumber::Peripheral(n) => {
                    self.hdls_periheral[n as usize] = Some(handler);
                }
            }
        }
    }
}
