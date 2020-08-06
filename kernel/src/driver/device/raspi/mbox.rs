use core::intrinsics::volatile_load;
use core::intrinsics::volatile_store;

use core::slice;

use super::graphics;
use super::memory::*;

use crate::aarch64::mmu;

// see https://github.com/raspberrypi/firmware/wiki/Mailbox-property-interface

const MBOX_REQUEST: u32 = 0;

// channels
const MBOX_CH_POWER: u8 = 0;
const MBOX_CH_FB: u8 = 1;
const MBOX_CH_VUART: u8 = 2;
const MBOX_CH_VCHIQ: u8 = 3;
const MBOX_CH_LEDS: u8 = 4;
const MBOX_CH_BTNS: u8 = 5;
const MBOX_CH_TOUCH: u8 = 6;
const MBOX_CH_COUNT: u8 = 7;
const MBOX_CH_PROP: u8 = 8;

const MBOX_TAG_GETFWVER: u32 = 0x00001;
const MBOX_TAG_GETBOARDMODEL: u32 = 0x10001;
const MBOX_TAG_GETBOARDREV: u32 = 0x10002;
const MBOX_TAG_GETBOARDMAC: u32 = 0x10003;
const MBOX_TAG_GETSERIAL: u32 = 0x10004;
const MBOX_TAG_GETMEM: u32 = 0x10005;
const MBOX_TAG_SETPOWER: u32 = 0x28001;
const MBOX_TAG_SETCLKRATE: u32 = 0x38002;
const MBOX_TAG_ALLOCFB: u32 = 0x40001; // allocate frame buffer
const MBOX_TAG_GETPITCH: u32 = 0x40008; // get pitch
const MBOX_TAG_SETPHY_WH: u32 = 0x48003; // set physical display's width and height
const MBOX_TAG_SETVIRT_WH: u32 = 0x48004; // set virtual display's width and height
const MBOX_TAG_SETDEPTH: u32 = 0x48005; // set depth
const MBOX_TAG_SETPIXELORDER: u32 = 0x48006; // set pixel order
const MBOX_TAG_SETVIRT_OFFSET: u32 = 0x48009; // set virtual display's offset
const MBOX_TAG_LAST: u32 = 0;

const VIDEOCORE_MBOX0: u32 = MMIO_BASE + 0x0000B880;
const MBOX0_READ: *mut u32 = (VIDEOCORE_MBOX0 + 0x0) as *mut u32;
const MBOX0_POLL: *mut u32 = (VIDEOCORE_MBOX0 + 0x10) as *mut u32;
const MBOX0_SENDER: *mut u32 = (VIDEOCORE_MBOX0 + 0x14) as *mut u32;
const MBOX0_STATUS: *mut u32 = (VIDEOCORE_MBOX0 + 0x18) as *mut u32;
const MBOX0_CONFIG: *mut u32 = (VIDEOCORE_MBOX0 + 0x1C) as *mut u32;
const MBOX0_WRITE: *mut u32 = (VIDEOCORE_MBOX0 + 0x20) as *mut u32;

const VIDEOCORE_MBOX1: u32 = MMIO_BASE + 0x0000B880;

const MBOX_RESPONSE: u32 = 0x80000000;
const MBOX_FULL: u32 = 0x80000000;
const MBOX_EMPTY: u32 = 0x40000000;

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
    let m = mmu::get_no_cache::<[u32; 8]>();

    // get the board's unique serial number with a mailbox call
    m[0] = 8 * 4; // length of the message
    m[1] = MBOX_REQUEST; // this is a request message
    m[2] = MBOX_TAG_GETSERIAL; // get serial number command
    m[3] = 8; // buffer size
    m[4] = 8;
    m[5] = 0; // clear output buffer
    m[6] = 0;
    m[7] = MBOX_TAG_LAST;

    if call(&mut (m[0]) as *mut u32, MBOX_CH_PROP) {
        let serial: u64 = m[5] as u64 | ((m[6] as u64) << 32);
        Some(serial)
    } else {
        None
    }
}

fn get_len7_u32(tag: u32) -> Option<u32> {
    let m = mmu::get_no_cache::<[u32; 7]>();
    m[0] = 7 * 4; // length of the message
    m[1] = MBOX_REQUEST; // this is a request message
    m[2] = tag; // get firmware version
    m[3] = 4; // buffer size
    m[4] = 4;
    m[5] = 0; // clear output buffer
    m[6] = MBOX_TAG_LAST;

    if call(&mut (m[0]) as *mut u32, MBOX_CH_PROP) {
        Some(m[5])
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

/// get memory size
pub fn get_memory() -> usize {
    match get_board_rev() {
        Some(rev) => {
            // https://www.raspberrypi.org/documentation/hardware/raspberrypi/revision-codes/README.md
            if (rev >> 23) & 1 == 0 {
                let m = mmu::get_no_cache::<[u32; 8]>();
                m[0] = 8 * 4; // length of the message
                m[1] = MBOX_REQUEST; // this is a request message
                m[2] = MBOX_TAG_GETMEM; // get memory
                m[3] = 8; // buffer size
                m[4] = 8;
                m[5] = 0; // clear output buffer
                m[6] = 0;
                m[7] = MBOX_TAG_LAST;

                if call(&mut (m[0]) as *mut u32, MBOX_CH_PROP) {
                    m[6] as usize
                } else {
                    256 * 1024 * 1024 // 256MiB
                }
            } else {
                match (rev >> 20) & 0b111 {
                    0 => 256 * 1024 * 1024,      // 256MiB
                    1 => 512 * 1024 * 1024,      // 512MiB
                    2 => 1024 * 1024 * 1024,     // 1GiB
                    3 => 2 * 1024 * 1024 * 1024, // 2GiB
                    4 => 4 * 1024 * 1024 * 1024, // 4GiB
                    _ => 256 * 1024 * 1024,      // 256MiB
                }
            }
        }
        _ => 256 * 1024 * 1024, // 256MiB
    }
}

pub fn set_uart_clock(clock: u32) {
    let m = mmu::get_no_cache::<[u32; 9]>();
    m[0] = 9 * 4;
    m[1] = MBOX_REQUEST;
    m[2] = MBOX_TAG_SETCLKRATE; // set clock rate
    m[3] = 12;
    m[4] = 8;
    m[5] = 2; // UART clock
    m[6] = clock;
    m[7] = 0; // clear turbo
    m[8] = MBOX_TAG_LAST;

    call(&mut (m[0]) as *mut u32, MBOX_CH_PROP);
}

/// power off a device
pub fn set_power_off(n: u32) {
    let m = mmu::get_no_cache::<[u32; 8]>();
    m[0] = 8 * 4; // length of the message
    m[1] = MBOX_REQUEST; // this is a request message
    m[2] = MBOX_TAG_SETPOWER; // get power state
    m[3] = 8; // buffer size
    m[4] = 8;
    m[5] = n; // device id
    m[6] = 0; // bit 0: off, bit 1: no wait
    m[7] = MBOX_TAG_LAST;

    call(&mut (m[0]) as *mut u32, MBOX_CH_PROP);
}

/// set display's setting
pub fn set_display(
    width_phy: u32,
    height_phy: u32,
    width_virt: u32,
    height_virt: u32,
    offset_x: u32,
    offset_y: u32,
) -> Option<graphics::Display> {
    let m = mmu::get_no_cache::<[u32; 35]>();
    m[0] = 35 * 4;
    m[1] = MBOX_REQUEST;

    m[2] = MBOX_TAG_SETPHY_WH; // set physical display's width and height
    m[3] = 8;
    m[4] = 8;
    m[5] = width_phy;
    m[6] = height_phy;

    m[7] = MBOX_TAG_SETVIRT_WH; // set virttual display's width and height
    m[8] = 8;
    m[9] = 8;
    m[10] = width_virt;
    m[11] = height_virt;

    m[12] = MBOX_TAG_SETVIRT_OFFSET; // set virtual display's offset
    m[13] = 8;
    m[14] = 8;
    m[15] = offset_x;
    m[16] = offset_y;

    m[17] = MBOX_TAG_SETDEPTH; // set depth
    m[18] = 4;
    m[19] = 4;
    m[20] = 32; // 32 bits per pixel

    m[21] = MBOX_TAG_SETPIXELORDER; // set pixel order
    m[22] = 4;
    m[23] = 4;
    m[24] = 1; // 0: BGR, 1: RGB

    m[25] = MBOX_TAG_ALLOCFB; // allocate frame buffer
    m[26] = 8;
    m[27] = 8;
    m[28] = 4096; // request: align 4096 bytes, responce: frame buffer base address
    m[29] = 0; // responce: frame buffer size

    m[30] = MBOX_TAG_GETPITCH; // get pitch
    m[31] = 4;
    m[32] = 4;
    m[33] = 0; // bytes per line

    m[34] = MBOX_TAG_LAST;

    if call(&mut (m[0]) as *mut u32, MBOX_CH_PROP) && m[20] == 32 && m[28] != 0 {
        let ptr = m[28] & 0x3FFFFFFF;
        let slice =
            unsafe { slice::from_raw_parts_mut(ptr as *mut u8, m[33] as usize * m[11] as usize) };
        Some(graphics::Display {
            size_phy: (m[5], m[6]),
            size_virt: (m[10], m[11]),
            offset: (m[15], m[16]),
            depth: m[20],
            pitch: m[33],
            ptr: ptr,
            buffer: slice,
        })
    } else {
        None
    }
}
