use crate::driver::setup;

pub(in crate::driver) struct Setup {}

impl setup::Setup for Setup {
    // TODO:
    // dummy
    fn early_platform_setup() {}

    #[cfg(feature = "raspi3")]
    fn platform_setup() {
        // enable UART0 interrupt
        //crate::int::enable_irq_num(int::int_rpi::IRQ_UART_INT);
        //uart::enable_recv_interrupt();
    }

    #[cfg(feature = "raspi4")]
    fn platform_setup() {}
}
