pub(in crate::driver) type IRQManager = int_rpi::IRQManager;
pub(in crate::driver) type IRQNumber = int_rpi::IRQNumber;

#[cfg(feature = "raspi4")]
use crate::driver::gic as int_rpi;

#[cfg(feature = "raspi3")]
pub(in crate::driver::device::raspi) mod int_rpi {
    use crate::{
        driver::device::raspi::memory,
        int::{self, IRQ},
    };
    use arr_macro::arr;
    use register::{mmio::*, register_structs};

    const LOCAL_INTERRUPT_CTRL_OFFSET: usize = 0xb200;
    const LOCAL_CTRL: usize = memory::MMIO_BASE + LOCAL_INTERRUPT_CTRL_OFFSET;

    // ARM interrupt registers
    // See page 112 of https://www.raspberrypi.org/app/uploads/2012/02/BCM2835-ARM-Peripherals.pdf
    //
    // Offset is 0x3f00b200 for Raspberry Pi 3
    register_structs! {
        #[allow(non_snake_case)]
        RegisterBlock {
            (0x000 => IRQ_Basic_Pending: ReadOnly<u32>),
            (0x004 => IRQ_Pending1: ReadOnly<u32>),
            (0x008 => IRQ_Pending2: ReadOnly<u32>),
            (0x00c => FIQ_Control: ReadWrite<u32>),
            (0x010 => Enable_IRQs1: ReadWrite<u32>),
            (0x014 => Enable_IRQs2: ReadWrite<u32>),
            (0x018 => Enable_Basic_IRQs: ReadWrite<u32>),
            (0x01c => Disable_IRQs1: ReadWrite<u32>),
            (0x020 => Disable_IRQs2: ReadWrite<u32>),
            (0x024 => Disable_Basic_IRQs2: ReadWrite<u32>),
            (0x028 => @END),
        }
    }

    const MAX_LOCAL_IRQ_NUMBER: usize = 11;
    const MAX_PERIPHERAL_IRQ_NUMBER: usize = 63;

    fn local_int_regs() -> *const RegisterBlock {
        LOCAL_CTRL as *const RegisterBlock
    }

    #[derive(Debug, PartialEq, Eq)]
    pub enum IRQNumber {
        Private(u8),
        Peripheral(u8),
    }

    pub(in crate::driver::device::raspi) const IRQ_SYSTEM_TIMER_MATCH1: IRQNumber =
        IRQNumber::Peripheral(1);
    pub(in crate::driver::device::raspi) const IRQ_SYSTEM_TIMER_MATCH3: IRQNumber =
        IRQNumber::Peripheral(3);
    pub(in crate::driver::device::raspi) const IRQ_USB_CONTROLLER: IRQNumber =
        IRQNumber::Peripheral(9);
    pub(in crate::driver::device::raspi) const IRQ_AUX_INT: IRQNumber = IRQNumber::Peripheral(29);
    pub(in crate::driver::device::raspi) const IRQ_I2C_SPI_SLV_INT: IRQNumber =
        IRQNumber::Peripheral(43);
    pub(in crate::driver::device::raspi) const IRQ_PWA0: IRQNumber = IRQNumber::Peripheral(45);
    pub(in crate::driver::device::raspi) const IRQ_PWA1: IRQNumber = IRQNumber::Peripheral(46);
    pub(in crate::driver::device::raspi) const IRQ_SMI: IRQNumber = IRQNumber::Peripheral(48);
    pub(in crate::driver::device::raspi) const IRQ_GPIP_INT0: IRQNumber = IRQNumber::Peripheral(49);
    pub(in crate::driver::device::raspi) const IRQ_GPIP_INT1: IRQNumber = IRQNumber::Peripheral(50);
    pub(in crate::driver::device::raspi) const IRQ_GPIP_INT2: IRQNumber = IRQNumber::Peripheral(51);
    pub(in crate::driver::device::raspi) const IRQ_I2C_INT: IRQNumber = IRQNumber::Peripheral(53);
    pub(in crate::driver::device::raspi) const IRQ_SPI_INT: IRQNumber = IRQNumber::Peripheral(54);
    pub(in crate::driver::device::raspi) const IRQ_PCM_INT: IRQNumber = IRQNumber::Peripheral(55);
    pub(in crate::driver::device::raspi) const IRQ_UART_INT: IRQNumber = IRQNumber::Peripheral(57);

    pub struct IRQManager {
        hdls_private: [Option<IRQ<IRQNumber>>; MAX_LOCAL_IRQ_NUMBER],
        hdls_periheral: [Option<IRQ<IRQNumber>>; MAX_PERIPHERAL_IRQ_NUMBER],
    }

    impl int::IRQManager for IRQManager {
        type IRQNumberType = IRQNumber;

        fn enable(&self, irq_num: Self::IRQNumberType) {
            match irq_num {
                IRQNumber::Private(_) => {
                    unimplemented!();
                }
                IRQNumber::Peripheral(n) => {
                    let regs = unsafe { &*local_int_regs() };
                    if n < 32 {
                        regs.Enable_IRQs1.set(1 << n);
                    } else {
                        regs.Enable_IRQs2.set(1 << (n - 32));
                    }
                }
            }
        }

        fn disable(&self, irq_num: Self::IRQNumberType) {
            match irq_num {
                IRQNumber::Private(_) => {
                    unimplemented!();
                }
                IRQNumber::Peripheral(n) => {
                    let regs = unsafe { &*local_int_regs() };
                    if n < 32 {
                        regs.Disable_IRQs1.set(1 << n);
                    } else {
                        regs.Disable_IRQs2.set(1 << (n - 32));
                    }
                }
            }
        }

        fn ack(&self, _irq_num: Self::IRQNumberType) {}

        fn handle(&self, irq_num: Self::IRQNumberType) {
            match irq_num {
                IRQNumber::Private(n) => {
                    let n = n as usize;
                    if let Some(f) = &self.hdls_private[n] {
                        f.handle(irq_num);
                    }
                }
                IRQNumber::Peripheral(n) => {
                    let n = n as usize;
                    assert!(n < MAX_PERIPHERAL_IRQ_NUMBER);
                    if let Some(f) = &self.hdls_periheral[n] {
                        f.handle(irq_num);
                    }
                }
            }
        }

        fn new() -> Self {
            let regs = unsafe { &*local_int_regs() };
            regs.Disable_Basic_IRQs2.set(0);
            regs.Disable_IRQs1.set(0);
            regs.Disable_IRQs2.set(0);

            IRQManager {
                hdls_private: arr![None; 11],
                hdls_periheral: arr![None; 63],
            }
        }

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
