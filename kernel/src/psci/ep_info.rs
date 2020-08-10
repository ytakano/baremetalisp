// Security state of the image.
pub const EP_SECURITY_MASK: usize = 0x1;
pub const EP_SECURITY_SHIFT: usize = 0;
pub const EP_SECURE: usize = 0x0;
pub const EP_NON_SECURE: usize = 0x1;

// Endianness of the image.
pub const EP_EE_MASK: usize = 0x2;
pub const EP_EE_SHIFT: usize = 1;
pub const EP_EE_LITTLE: usize = 0x0;
pub const EP_EE_BIG: usize = 0x2;

// Enable or disable access to the secure timer from secure images.
pub const EP_ST_MASK: usize = 0x4;
pub const EP_ST_SHIFT: usize = 2;
pub const EP_ST_DISABLE: usize = 0x0;
pub const EP_ST_ENABLE: usize = 0x4;

// Param header types
pub const PARAM_EP: u8 = 0x01;
pub const PARAM_IMAGE_BINARY: u8 = 0x02;
pub const PARAM_BL31: u8 = 0x03;
pub const PARAM_BL_LOAD_INFO: u8 = 0x04;
pub const PARAM_BL_PARAMS: u8 = 0x05;
pub const PARAM_PSCI_LIB_ARGS: u8 = 0x06;
pub const PARAM_SP_IMAGE_BOOT_INFO: u8 = 0x07;

// Param header version
pub const PARAM_VERSION_1: u8 = 0x01;
pub const PARAM_VERSION_2: u8 = 0x02;

/// This structure provides version information and the size of the
/// structure, attributes for the structure it represents
#[repr(C)]
pub struct ParamHeader {
    pub htype: u8,   // type of the structure
    pub version: u8, // version of this structure
    pub size: u16,   // size of this structure in bytes
    pub attr: u32,   // attributes: unused bits SBZ
}

#[repr(C)]
pub struct Aapcs64Params {
    pub arg0: u64,
    pub arg1: u64,
    pub arg2: u64,
    pub arg3: u64,
    pub arg4: u64,
    pub arg5: u64,
    pub arg6: u64,
    pub arg7: u64,
}

impl Aapcs64Params {
    pub fn new() -> Aapcs64Params {
        Aapcs64Params {
            arg0: 0,
            arg1: 0,
            arg2: 0,
            arg3: 0,
            arg4: 0,
            arg5: 0,
            arg6: 0,
            arg7: 0,
        }
    }
}

// AArch32
// #[repr(C)]
// pub struct Aapcs32Params {
//     pub arg0: u32,
//     pub arg1: u32,
//     pub arg2: u32,
//     pub arg3: u32,
// }

// This structure represents the superset of information needed while
// switching exception levels. The only two mechanisms to do so are
// ERET & SMC. Security state is indicated using bit zero of header
// attribute
// NOTE: BL1 expects entrypoint followed by spsr at an offset from the start
// of this structure defined by the macro `ENTRY_POINT_INFO_PC_OFFSET` while
// processing SMC to jump to BL31.
pub struct EntryPointInfo {
    pub h: ParamHeader,
    pub pc: usize,
    pub spsr: u32,

    // AArch64
    pub args: Aapcs64Params,
    // AArch32
    // pub lr_svc: usize,
    // pub args: Aapcs32Params,
}
