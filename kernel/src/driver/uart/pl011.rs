use crate::{driver::uart::UART, mmio_rw_base};

const CR_RXE: u32 = 1 << 9;
const CR_TXE: u32 = 1 << 8;
const CR_EN: u32 = 1;

const ICR_ALL_CLEAR: u32 = 0x7ff;

const LCRH_WLEN_8BITS: u32 = 0b11 << 5; // Word length (8bits)
const LCRH_FEN_FIFO: u32 = 1 << 4; // Enable FIFOs

const IFLS_RXIFLSEL_1_8: u32 = 0b000;
const IFLS_RXIFLSEL_1_4: u32 = 0b001 << 3;
const IFLS_RXIFLSEL_1_2: u32 = 0b010 << 3;
const IFLS_RXIFLSEL_3_4: u32 = 0b011 << 3;
const IFLS_RXIFLSEL_7_8: u32 = 0b100 << 3;

const IMSC_RXIM: u32 = 1 << 4;

pub struct PL011 {
    base: usize,
}

impl PL011 {
    mmio_rw_base!(0x000 => uart0_dr<u32>);
    mmio_rw_base!(0x004 => uart0_rsrecr<u32>);
    mmio_rw_base!(0x018 => uart0_fr<u32>);
    mmio_rw_base!(0x020 => uart0_ilpr<u32>);
    mmio_rw_base!(0x024 => uart0_ibrd<u32>);
    mmio_rw_base!(0x028 => uart0_fbrd<u32>);
    mmio_rw_base!(0x02c => uart0_lcrh<u32>);
    mmio_rw_base!(0x030 => uart0_cr<u32>);
    mmio_rw_base!(0x034 => uart0_ifls<u32>);
    mmio_rw_base!(0x038 => uart0_imsc<u32>);
    mmio_rw_base!(0x03c => uart0_ris<u32>);
    mmio_rw_base!(0x040 => uart0_mis<u32>);
    mmio_rw_base!(0x044 => uart0_icr<u32>);
    mmio_rw_base!(0x048 => uart0_dmacr<u32>);
}

impl UART for PL011 {
    fn new(base: usize) -> Self {
        PL011 { base }
    }

    /// Initialiaze UART0 for serial console.
    /// Set baud rate and characteristics (8N1) and map to GPIO 14 (Tx) and 15 (Rx).
    /// 8N1 stands for "eight data bits, no parity, one stop bit".
    fn init(&self, uart_clock: usize, baudrate: usize) {
        let bauddiv: u32 = ((1000 * uart_clock) / (16 * baudrate)) as u32;
        let ibrd: u32 = bauddiv / 1000;
        let fbrd: u32 = ((bauddiv - ibrd * 1000) * 64 + 500) / 1000;
        self.uart0_icr().write(ICR_ALL_CLEAR); // clear interrupts
        self.uart0_ibrd().write(ibrd);
        self.uart0_fbrd().write(fbrd);

        self.uart0_lcrh().write(LCRH_WLEN_8BITS | LCRH_FEN_FIFO); // 8n1, FIFO
        self.uart0_ifls().write(IFLS_RXIFLSEL_1_4); // RX FIFO fill level at 1/4
    }

    /// send a character to serial console
    fn send(&self, c: u32) {
        // wait until we can send
        unsafe { asm!("nop;") };
        while self.uart0_fr().read() & 0x20 != 0 {
            unsafe { asm!("nop;") };
        }

        // write the character to the buffer
        self.uart0_dr().write(c);
    }

    fn recv(&self) -> u32 {
        // wait until something is in the buffer
        unsafe { asm!("nop;") };
        while self.uart0_fr().read() & 0x10 != 0 {
            unsafe { asm!("nop;") };
        }

        self.uart0_dr().read()
    }

    fn enable_recv_interrupt(&self) {
        self.uart0_imsc().setbits(IMSC_RXIM);
    }

    fn disable_recv_interrupt(&self) {
        self.uart0_imsc().clrbits(IMSC_RXIM);
    }

    fn on(&self) {
        self.uart0_cr().write(0); // turn off UART0
    }

    fn off(&self) {
        self.uart0_cr().write(CR_EN | CR_RXE | CR_TXE); // enable, Rx, Tx
    }
}
