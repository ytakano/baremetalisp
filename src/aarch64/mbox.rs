use core::intrinsics::volatile_store;
use core::intrinsics::volatile_load;

use super::memory::*;
use super::uart;

pub const MBOX_REQUEST: u32 = 0;

// channels
pub const MBOX_CH_POWER: u8 = 0;
pub const MBOX_CH_FB:    u8 = 1;
pub const MBOX_CH_VUART: u8 = 2;
pub const MBOX_CH_VCHIQ: u8 = 3;
pub const MBOX_CH_LEDS:  u8 = 4;
pub const MBOX_CH_BTNS:  u8 = 5;
pub const MBOX_CH_TOUCH: u8 = 6;
pub const MBOX_CH_COUNT: u8 = 7;
pub const MBOX_CH_PROP:  u8 = 8;

pub const MBOX_TAG_GETSERIAL:  u32 = 0x10004;
pub const MBOX_TAG_SETCLKRATE: u32 = 0x38002;
pub const MBOX_TAG_LAST:       u32 = 0;

pub const VIDEOCORE_MBOX: u32 = MMIO_BASE + 0x0000B880;
pub const MBOX_READ:      *mut u32 = (VIDEOCORE_MBOX + 0x0 ) as *mut u32;
pub const MBOX_POLL:      *mut u32 = (VIDEOCORE_MBOX + 0x10) as *mut u32;
pub const MBOX_SENDER:    *mut u32 = (VIDEOCORE_MBOX + 0x14) as *mut u32;
pub const MBOX_STATUS:    *mut u32 = (VIDEOCORE_MBOX + 0x18) as *mut u32;
pub const MBOX_CONFIG:    *mut u32 = (VIDEOCORE_MBOX + 0x1C) as *mut u32;
pub const MBOX_WRITE:     *mut u32 = (VIDEOCORE_MBOX + 0x20) as *mut u32;
pub const MBOX_RESPONSE:  u32 = 0x80000000;
pub const MBOX_FULL:      u32 = 0x80000000;
pub const MBOX_EMPTY:     u32 = 0x40000000;

pub fn call(ptr: *mut u32, ch: u8) -> bool {
    let r = ptr as u64;
    if r & 0xF != 0 || r > 0xFFFFFFFF {
        return false;
    }

    let r: u32 = ((r & !0xF) | (ch as u64 & 0xF)) as u32;

    // wait until we can write to the mailbox
    unsafe { asm!("nop;") };
    while unsafe { volatile_load(MBOX_STATUS) } & MBOX_FULL > 0 {
        unsafe { asm!("nop;") };
    };

    // write the address of our message to the mailbox with channel identifier
    unsafe { volatile_store(MBOX_WRITE, r) };

    // now wait for the response
    let ptr1 = ((ptr as u64) + 4) as *mut u32;
    loop {
        // is there a response?
        unsafe { asm!("nop;") };
        while unsafe { volatile_load(MBOX_STATUS) } & MBOX_EMPTY > 0 {
            unsafe { asm!("nop;") };
        };

        if r == unsafe { volatile_load(MBOX_READ) } {
            return unsafe { volatile_load(ptr1) } == MBOX_RESPONSE;
        }
    }
}

#[repr(align(16))]
struct Mbox<T>(T);

pub fn get_serial() -> Option<u64> {
    // get the board's unique serial number with a mailbox call
    let mut m = Mbox::<[u32; 8]>([
        8 * 4,              // length of the message
        MBOX_REQUEST,       // this is a request message
        MBOX_TAG_GETSERIAL, // get serial number command
        8,                  // buffer size
        8,
        0,                  // clear output buffer
        0,
        MBOX_TAG_LAST
    ]);

    if call(&mut(m.0[0]) as *mut u32, MBOX_CH_PROP) {
        let serial: u64 = m.0[5] as u64 | ((m.0[6] as u64) << 32);
        Some(serial)
    } else {
        None
    }
}

pub fn set_uart_clock(clock: u32) -> () {
    let mut m = Mbox::<[u32; 9]>([
        9 * 4,
        MBOX_REQUEST,
        MBOX_TAG_SETCLKRATE, // set clock rate
        12,
        8,
        2,     // UART clock
        clock,
        0,     // clear turbo
        MBOX_TAG_LAST
    ]);

    call(&mut(m.0[0]) as *mut u32, MBOX_CH_PROP);
}