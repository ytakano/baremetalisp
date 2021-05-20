use super::memory::*;
use crate::driver::{delays, uart::UART};
use core::ptr::read_volatile;
use register::{mmio::*, register_structs};

const UART0_FR: *mut u32 = (MMIO_BASE + 0x00201018) as *mut u32;

pub(in crate::driver) struct RaspiUART {}

register_structs! {
    #[allow(non_snake_case)]
    UART0Registers {
        (0x000 => DR: ReadWrite<u32>),
        (0x004 => RSRECR: ReadWrite<u32>),
        (0x018 => FR: ReadWrite<u32>),
        (0x020 => ILPR: ReadWrite<u32>),
        (0x024 => IBRD: ReadWrite<u32>),
        (0x028 => FBRD: ReadWrite<u32>),
        (0x02c => LCRH: ReadWrite<u32>),
        (0x030 => CR: ReadWrite<u32>),
        (0x034 => IFLS: ReadWrite<u32>),
        (0x038 => IMSC: ReadWrite<u32>),
        (0x03c => RIS: ReadWrite<u32>),
        (0x040 => MIS: ReadWrite<u32>),
        (0x044 => ICR: ReadWrite<u32>),
        (0x048 => DMACR: ReadWrite<u32>),
        (0x080 => ITCR: ReadWrite<u32>),
        (0x084 => ITIP: ReadWrite<u32>),
        (0x088 => ITOP: ReadWrite<u32>),
        (0x08c => TDR: ReadWrite<u32>),
        (0x0a4 => @END),
    }
}

fn uart0_registers() -> *const UART0Registers {
    (MMIO_BASE + 0x00201000) as *const UART0Registers
}

impl UART for RaspiUART {
    /// Initialiaze UART0 for serial console.
    /// Set baud rate and characteristics (8N1) and map to GPIO 14 (Tx) and 15 (Rx).
    /// 8N1 stands for "eight data bits, no parity, one stop bit".
    fn init(uart_clock: usize, baudrate: usize) {
        let gpio = unsafe { &*gpio_registers() };
        let uart0 = unsafe { &*uart0_registers() };

        uart0.CR.set(0); // turn off UART0

        // map UART1 to GPIO pins
        let mut r = gpio.GPFSEL1.get();
        r &= !((7 << 12) | (7 << 15)); // gpio14, gpio15
        r |= (4 << 12) | (4 << 15); // alt0

        // enable pins 14 and 15
        gpio.GPFSEL1.set(r);
        gpio.GPPUD.set(0);

        delays::wait_cycles(150);

        gpio.GPPUDCLK0.set((1 << 14) | (1 << 15));

        delays::wait_cycles(150);

        let bauddiv: u32 = ((1000 * uart_clock) / (16 * baudrate)) as u32;
        let ibrd: u32 = bauddiv / 1000;
        let fbrd: u32 = ((bauddiv - ibrd * 1000) * 64 + 500) / 1000;

        gpio.GPPUDCLK0.set(0); // flush GPIO setup
        uart0.ICR.set(0x7ff); // clear interrupts
        uart0.IBRD.set(ibrd);
        uart0.FBRD.set(fbrd);
        uart0.LCRH.set(0b11 << 5); // 8n1
        uart0.CR.set(0x301); // enable Tx, Rx, FIFO
    }

    /// send a character to serial console
    fn send(c: u32) {
        let uart0 = unsafe { &*uart0_registers() };

        // wait until we can send
        unsafe { asm!("nop;") };
        while unsafe { read_volatile(UART0_FR) } & 0x20 != 0 {
            unsafe { asm!("nop;") };
        }

        // write the character to the buffer
        uart0.DR.set(c)
    }

    fn recv() -> u32 {
        let uart0 = unsafe { &*uart0_registers() };

        // wait until something is in the buffer
        unsafe { asm!("nop;") };
        while unsafe { read_volatile(UART0_FR) } & 0x10 != 0 {
            unsafe { asm!("nop;") };
        }

        uart0.DR.get()
    }

    fn enable_recv_interrupt() {
        let uart0 = unsafe { &*uart0_registers() };
        let imsc = uart0.IMSC.get();
        uart0.IMSC.set(imsc | (1 << 4));
    }

    fn disable_recv_interrupt() {
        let uart0 = unsafe { &*uart0_registers() };
        let imsc = uart0.IMSC.get();
        uart0.IMSC.set(imsc & !(1 << 4));
    }
}
