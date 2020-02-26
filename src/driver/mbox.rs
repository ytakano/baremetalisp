use core::intrinsics::volatile_store;
use core::intrinsics::volatile_load;

use core::slice;

use super::memory::*;
use super::graphics;

// see https://github.com/raspberrypi/firmware/wiki/Mailbox-property-interface

const MBOX_REQUEST: u32 = 0;

// channels
const MBOX_CH_POWER: u8 = 0;
const MBOX_CH_FB:    u8 = 1;
const MBOX_CH_VUART: u8 = 2;
const MBOX_CH_VCHIQ: u8 = 3;
const MBOX_CH_LEDS:  u8 = 4;
const MBOX_CH_BTNS:  u8 = 5;
const MBOX_CH_TOUCH: u8 = 6;
const MBOX_CH_COUNT: u8 = 7;
const MBOX_CH_PROP:  u8 = 8;

const MBOX_TAG_GETFWVER:       u32 = 0x00001;
const MBOX_TAG_GETBOARDMODEL:  u32 = 0x10001;
const MBOX_TAG_GETBOARDREV:    u32 = 0x10002;
const MBOX_TAG_GETBOARDMAC:    u32 = 0x10003;
const MBOX_TAG_GETSERIAL:      u32 = 0x10004;
const MBOX_TAG_GETMEM:         u32 = 0x10005;
const MBOX_TAG_SETPOWER:       u32 = 0x28001;
const MBOX_TAG_SETCLKRATE:     u32 = 0x38002;
const MBOX_TAG_ALLOCFB:        u32 = 0x40001; // allocate frame buffer
const MBOX_TAG_GETPITCH:       u32 = 0x40008; // get pitch
const MBOX_TAG_SETPHY_WH:      u32 = 0x48003; // set physical display's width and height
const MBOX_TAG_SETVIRT_WH:     u32 = 0x48004; // set virtual display's width and height
const MBOX_TAG_SETDEPTH:       u32 = 0x48005; // set depth
const MBOX_TAG_SETPIXELORDER:  u32 = 0x48006; // set pixel order
const MBOX_TAG_SETVIRT_OFFSET: u32 = 0x48009; // set virtual display's offset
const MBOX_TAG_LAST:           u32 = 0;

const VIDEOCORE_MBOX0: u32 = MMIO_BASE + 0x0000B880;
const MBOX0_READ:      *mut u32 = (VIDEOCORE_MBOX0 + 0x0 ) as *mut u32;
const MBOX0_POLL:      *mut u32 = (VIDEOCORE_MBOX0 + 0x10) as *mut u32;
const MBOX0_SENDER:    *mut u32 = (VIDEOCORE_MBOX0 + 0x14) as *mut u32;
const MBOX0_STATUS:    *mut u32 = (VIDEOCORE_MBOX0 + 0x18) as *mut u32;
const MBOX0_CONFIG:    *mut u32 = (VIDEOCORE_MBOX0 + 0x1C) as *mut u32;
const MBOX0_WRITE:     *mut u32 = (VIDEOCORE_MBOX0 + 0x20) as *mut u32;

const VIDEOCORE_MBOX1: u32 = MMIO_BASE + 0x0000B880;

const MBOX_RESPONSE:  u32 = 0x80000000;
const MBOX_FULL:      u32 = 0x80000000;
const MBOX_EMPTY:     u32 = 0x40000000;

pub fn call(ptr: *mut u32, ch: u8) -> bool {
    let r = ptr as u64;
    if r & 0xF != 0 || r > 0xFFFFFFFF {
        return false;
    }

    let r: u32 = ((r & !0xF) | (ch as u64 & 0xF)) as u32;

    // wait until we can write to the mailbox
    unsafe { asm!("nop;") };
    while unsafe { volatile_load(MBOX0_STATUS) } & MBOX_FULL > 0 {
        unsafe { asm!("nop;") };
    }

    // write the address of our message to the mailbox with channel identifier
    unsafe { volatile_store(MBOX0_WRITE, r) };

    // now wait for the response
    let ptr1 = ((ptr as u64) + 4) as *mut u32;
    loop {
        // is there a response?
        unsafe { asm!("nop;") };
        while unsafe { volatile_load(MBOX0_STATUS) } & MBOX_EMPTY > 0 {
            unsafe { asm!("nop;") };
        }

        if r == unsafe { volatile_load(MBOX0_READ) } {
            return unsafe { volatile_load(ptr1) } == MBOX_RESPONSE;
        }
    }
}

#[repr(align(16))]
struct Mbox<T>(T);

/// get board's serial number
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

fn get_len7_u32(tag: u32) -> Option<u32> {
    let mut m = Mbox::<[u32; 7]>([
        7 * 4,        // length of the message
        MBOX_REQUEST, // this is a request message
        tag,          // get firmware version
        4,            // buffer size
        4,
        0,            // clear output buffer
        MBOX_TAG_LAST
    ]);

    if call(&mut(m.0[0]) as *mut u32, MBOX_CH_PROP) {
        Some(m.0[5])
    } else {
        None
    }
}

/// get firmware version
pub fn get_firmware_version() -> Option<u32> {
    get_len7_u32(MBOX_TAG_GETFWVER)
}

/// get board model
pub fn get_board_model() -> Option<u32> {
    get_len7_u32(MBOX_TAG_GETBOARDMODEL)
}

/// get board revision
pub fn get_board_rev() -> Option<u32> {
    get_len7_u32(MBOX_TAG_GETBOARDREV)
}

/// get ARM memory
pub fn get_memory() -> Option<(u32, u32)> {
    let mut m = Mbox::<[u32; 8]>([
        8 * 4,           // length of the message
        MBOX_REQUEST,    // this is a request message
        MBOX_TAG_GETMEM, // get memory
        8,               // buffer size
        8,
        0,               // clear output buffer
        0,
        MBOX_TAG_LAST
    ]);

    if call(&mut(m.0[0]) as *mut u32, MBOX_CH_PROP) {
        Some((m.0[5], m.0[6]))
    } else {
        None
    }
}

pub fn set_uart_clock(clock: u32) {
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

/// power off a device
pub fn set_power_off(n: u32) {
    let mut m = Mbox::<[u32; 8]>([
        8 * 4,             // length of the message
        MBOX_REQUEST,      // this is a request message
        MBOX_TAG_SETPOWER, // get power state
        8,                 // buffer size
        8,
        n,                 // device id
        0,                 // bit 0: off, bit 1: no wait
        MBOX_TAG_LAST
    ]);

    call(&mut(m.0[0]) as *mut u32, MBOX_CH_PROP);
}

/// set display's setting
pub fn set_display(width_phy: u32, height_phy: u32, width_virt: u32, height_virt: u32,
                   offset_x: u32, offset_y: u32) -> Option<graphics::Display> {
    let mut m = Mbox::<[u32; 35]>([
        35 * 4,
        MBOX_REQUEST,

        MBOX_TAG_SETPHY_WH,      // set physical display's width and height
        8,
        8,
        width_phy,
        height_phy,

        MBOX_TAG_SETVIRT_WH,     // set virttual display's width and height
        8,
        8,
        width_virt,
        height_virt,

        MBOX_TAG_SETVIRT_OFFSET, // set virtual display's offset
        8,
        8,
        offset_x,
        offset_y,

        MBOX_TAG_SETDEPTH,       // set depth
        4,
        4,
        32,   // 32 bits per pixel

        MBOX_TAG_SETPIXELORDER,  // set pixel order
        4,
        4,
        1,    // 0: BGR, 1: RGB

        MBOX_TAG_ALLOCFB,        // allocate frame buffer
        8,
        8,
        4096, // request: align 4096 bytes, responce: frame buffer base address
        0,    // responce: frame buffer size

        MBOX_TAG_GETPITCH,       // get pitch
        4,
        4,
        0,    // bytes per line

        MBOX_TAG_LAST
    ]);


    if call(&mut(m.0[0]) as *mut u32, MBOX_CH_PROP) && m.0[20] == 32 && m.0[28] != 0 {
        let ptr = m.0[28] & 0x3FFFFFFF;
        let slice = unsafe {
            slice::from_raw_parts_mut(ptr as *mut u8,
                                  m.0[33] as usize * m.0[11] as usize) };
        Some(graphics::Display{
            size_phy: (m.0[5], m.0[6]),
            size_virt: (m.0[10], m.0[11]),
            offset: (m.0[15], m.0[16]),
            depth: m.0[20],
            pitch: m.0[33],
            ptr: ptr,
            buffer: slice
        })
    } else {
        None
    }
}