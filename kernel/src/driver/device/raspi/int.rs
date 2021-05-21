pub(in crate::driver) type IRQManager = int_rpi::IRQManager;
pub(in crate::driver) type IRQNumber = int_rpi::IRQNumber;

#[cfg(feature = "raspi4")]
use crate::driver::gic as int_rpi;

#[cfg(feature = "raspi3")]
pub(in crate::driver::device::raspi) mod int_rpi {
    use crate::{
        driver::device::raspi::memory,
        int::{self, IRQ},
        mmio_r, mmio_rw,
    };
    use arr_macro::arr;

    const LOCAL_INTERRUPT_CTRL_OFFSET: usize = 0xb200;
    const LOCAL_CTRL: usize = memory::MMIO_BASE + LOCAL_INTERRUPT_CTRL_OFFSET;

    // ARM interrupt registers
    // See page 112 of https://www.raspberrypi.org/app/uploads/2012/02/BCM2835-ARM-Peripherals.pdf
    //
    // Offset is 0x3f00b200 for Raspberry Pi 3
    mmio_r! (LOCAL_CTRL         => irq_basic_pending<u32>);
    mmio_r! (LOCAL_CTRL + 0x004 => irq_pending1<u32>);
    mmio_r! (LOCAL_CTRL + 0x008 => irq_pending2<u32>);
    mmio_rw!(LOCAL_CTRL + 0x00c => fiq_control<u32>);
    mmio_rw!(LOCAL_CTRL + 0x010 => enable_irqs1<u32>);
    mmio_rw!(LOCAL_CTRL + 0x014 => enable_irqs2<u32>);
    mmio_rw!(LOCAL_CTRL + 0x018 => enable_basic_irqs<u32>);
    mmio_rw!(LOCAL_CTRL + 0x01c => disable_irqs1<u32>);
    mmio_rw!(LOCAL_CTRL + 0x020 => disable_irqs2<u32>);
    mmio_rw!(LOCAL_CTRL + 0x024 => disable_basic_irqs<u32>);

    const MAX_LOCAL_IRQ_NUMBER: usize = 11;
    const MAX_PERIPHERAL_IRQ_NUMBER: usize = 63;

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
                    if n < 32 {
                        enable_irqs1().write(1 << n);
                    } else {
                        enable_irqs2().write(1 << (n - 32));
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
                    if n < 32 {
                        disable_irqs1().write(1 << n);
                    } else {
                        disable_irqs2().write(1 << (n - 32));
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
            disable_basic_irqs().write(0);
            disable_irqs1().write(0);
            disable_irqs2().write(0);

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
