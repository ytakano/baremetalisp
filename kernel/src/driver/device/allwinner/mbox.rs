use core::ptr::{read_volatile, write_volatile};

use super::memory::SUNXI_MSGBOX_BASE;
use crate::aarch64::lock;
use crate::bits::genmask32;
use crate::driver::delays::wait_microsec;

static mut LOCK: lock::BakeryTicket = lock::BakeryTicket::new();

const REMOTE_IRQ_STAT_REG: u32 = 0x0050;
const LOCAL_IRQ_STAT_REG: u32 = 0x0070;

const RX_CHAN: u32 = 1;
const TX_CHAN: u32 = 0;
const MHU_TIMEOUT_ITERS: u32 = 10000;
const MHU_TIMEOUT_DELAY: u32 = 10;

pub(crate) fn rx_irx(n: u32) -> u32 {
    1 << (2 * n)
}

fn msg_stat_reg(n: u32) -> u32 {
    0x0140 + 4 * n
}

fn msg_stat_mask() -> u32 {
    genmask32(2, 0)
}

fn msg_data_reg(n: u32) -> u32 {
    0x0180 + 4 * (n)
}

pub(crate) fn sunxi_msgbox_last_tx_done(chan: u32) -> bool {
    let addr = (SUNXI_MSGBOX_BASE + REMOTE_IRQ_STAT_REG) as *mut u32;
    let stat = unsafe { read_volatile(addr) };
    stat & rx_irx(chan) == 0
}

pub(crate) fn sunxi_msgbox_peek_data(chan: u32) -> bool {
    let addr = (SUNXI_MSGBOX_BASE + msg_stat_reg(chan)) as *mut u32;
    (unsafe { read_volatile(addr) } & msg_stat_mask()) != 0
}

pub struct SecureMsgLock<'a> {
    lock: lock::BakeryLock<'a>,
}

impl<'a> SecureMsgLock<'a> {
    pub fn new() -> SecureMsgLock<'a> {
        let mut timeout = MHU_TIMEOUT_ITERS;
        let lock = SecureMsgLock {
            lock: unsafe { LOCK.lock() },
        };

        // Wait for all previous messages to be acknowledged.
        while !sunxi_msgbox_last_tx_done(TX_CHAN) && timeout > 0 {
            wait_microsec(MHU_TIMEOUT_DELAY);
            timeout -= 1;
        }

        lock
    }
}

impl<'a> Drop for SecureMsgLock<'a> {
    fn drop(&mut self) {
        let addr = (SUNXI_MSGBOX_BASE + LOCAL_IRQ_STAT_REG) as *mut u32;
        unsafe {
            write_volatile(addr, rx_irx(RX_CHAN));
        }
    }
}

pub(crate) fn mhu_secure_message_wait() -> u32 {
    let mut timeout = MHU_TIMEOUT_ITERS;

    // Wait for a message from the SCP.
    while !sunxi_msgbox_peek_data(RX_CHAN) && timeout > 0 {
        wait_microsec(MHU_TIMEOUT_DELAY);
        timeout -= 1;
    }

    // Return the most recent message in the FIFO.
    let addr = (SUNXI_MSGBOX_BASE + msg_data_reg(RX_CHAN)) as *mut u32;

    let mut msg = 0;
    while sunxi_msgbox_peek_data(RX_CHAN) {
        msg = unsafe { read_volatile(addr) };
    }

    msg
}

pub(crate) fn mhu_secure_message_send(slot_id: u32) {
    let addr = (SUNXI_MSGBOX_BASE + msg_data_reg(TX_CHAN)) as *mut u32;
    unsafe {
        write_volatile(addr, 1 << slot_id);
    }
}
