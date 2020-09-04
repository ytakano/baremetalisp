use core::ptr::{copy, write_volatile};

use crate::aarch64::cpu;
use crate::driver::memory::CSS_SCP_COM_SHARED_MEM_BASE;
use crate::driver::mhu;

use core::mem::size_of;

const SCPI_SHARED_MEM_SCP_TO_AP: u32 = CSS_SCP_COM_SHARED_MEM_BASE;
const SCPI_SHARED_MEM_AP_TO_SCP: u32 = CSS_SCP_COM_SHARED_MEM_BASE + 0x100;
const SCPI_CMD_PAYLOAD_AP_TO_SCP: u32 = SCPI_SHARED_MEM_AP_TO_SCP + size_of::<ScpiCmd>() as u32;

const SCPI_MHU_SLOT_ID: u32 = 0;

enum ScpiSet {
    ScpiSetNormal = 0, // Normal SCPI commands
    ScpiSetExtended,   // Extended SCPI commands
}

#[derive(PartialEq, Clone, Copy)]
pub enum ScpiResult {
    Ok = 0,    // Success
    EParam,    // Invalid parameter(s)
    EAlign,    // Invalid alignment
    ESize,     // Invalid size
    EHandler,  // Invalid handler or callback
    EAccess,   // Invalid access or permission denied
    ERange,    // Value out of range
    ETimeout,  // Time out has ocurred
    ENomem,    // Invalid memory area or pointer
    EPwrstate, // Invalid power state
    ESupport,  // Feature not supported or disabled
    EDevice,   // Device error
    EBusy,     // Device is busy
}

impl ScpiResult {
    pub fn from_u32(n: u32) -> ScpiResult {
        match n {
            0 => ScpiResult::Ok,
            1 => ScpiResult::EParam,
            2 => ScpiResult::EAlign,
            3 => ScpiResult::ESize,
            4 => ScpiResult::EHandler,
            5 => ScpiResult::EAccess,
            6 => ScpiResult::ERange,
            7 => ScpiResult::ETimeout,
            8 => ScpiResult::ENomem,
            9 => ScpiResult::EPwrstate,
            10 => ScpiResult::ESupport,
            11 => ScpiResult::EDevice,
            12 => ScpiResult::EBusy,
            _ => ScpiResult::EDevice,
        }
    }
}

enum ScpiCommand {
    ScpReady = 0x01,
    SetCssPowerState = 0x03,
    GetCssPowerState = 0x04,
    SysPowerState = 0x05,
}

pub enum ScpiPowerState {
    PowerOn = 0,
    PowerRetention = 1,
    PowerOff = 3,
}

pub enum ScpiSystemState {
    Shutdown = 0,
    Reboot = 1,
    Reset = 2,
}

#[repr(C)]
pub struct ScpiCmd {
    id: u32,        // Command ID
    set: u32,       // Set ID. Identifies whether this is a standard or extended command.
    sender: u32,    // Sender ID to match a reply. The value is sender specific.
    size: u32,      // Size of the payload in bytes (0 - 511)
    _reserved: u32, // Reseved
    status: u32,    // Status indicating the success of a command. See the enum below.
}

impl ScpiCmd {
    fn new() -> ScpiCmd {
        ScpiCmd {
            id: 0,
            set: 0,
            sender: 0,
            size: 0,
            _reserved: 0,
            status: 0,
        }
    }

    fn set_id(&mut self, id: ScpiCommand) {
        self.id = id as u32;
    }

    fn set_set(&mut self, set: ScpiSet) {
        self.set = set as u32;
    }
}

pub fn wait_ready() -> bool {
    let mut cmd = ScpiCmd::new();

    {
        // Get a message from the SCP
        mhu::SecureMsgLock::new();
        if !secure_message_receive(&mut cmd) {
            // If no message was received, don't send a response
            return false;
        }
    }

    // We are expecting 'SCP Ready', produce correct error if it's not
    let status = if cmd.id != ScpiCommand::ScpReady as u32 {
        ScpiResult::ESupport
    } else if cmd.size != 0 {
        ScpiResult::ESize
    } else {
        ScpiResult::Ok
    };

    // Send our response back to SCP.
    // We are using the same SCPI header, just update the status field.
    cmd.status = status as u32;
    {
        mhu::SecureMsgLock::new();
        unsafe {
            copy(
                &cmd as *const ScpiCmd,
                SCPI_SHARED_MEM_AP_TO_SCP as *mut ScpiCmd,
                1,
            );
        }

        secure_message_send(0);
    }

    status == ScpiResult::Ok
}

pub fn secure_message_send(_payload_size: usize) {
    // Ensure that any write to the SCPI payload area is seen by SCP before
    // we write to the MHU register. If these 2 writes were reordered by
    // the CPU then SCP would read stale payload data
    cpu::dmb_st();

    mhu::mhu_secure_message_send(SCPI_MHU_SLOT_ID);
}

pub fn secure_message_receive(cmd: &mut ScpiCmd) -> bool {
    let mhu_status = mhu::mhu_secure_message_wait();

    // Expect an SCPI message, reject any other protocol
    if mhu_status != (1 << SCPI_MHU_SLOT_ID) {
        return false;
    }

    // Ensure that any read to the SCPI payload area is done after reading
    // the MHU register. If these 2 reads were reordered then the CPU would
    // read invalid payload data
    cpu::dmb_ld();
    unsafe {
        copy(
            SCPI_SHARED_MEM_SCP_TO_AP as *const ScpiCmd,
            cmd as *mut ScpiCmd,
            1,
        );
    }

    true
}

pub fn set_css_power_state(
    mpidr: usize,
    cpu_state: ScpiPowerState,
    cluster_state: ScpiPowerState,
    css_state: ScpiPowerState,
) {
    let state: u32 = (mpidr & 0xf) as u32 // CPU ID
        | ((mpidr & 0xf00) >> 4) as u32   // Cluster ID
        | (cpu_state as u32) << 8
        | (css_state as u32) << 16
        | (cluster_state as u32) << 12;

    // Populate the command header
    let mut cmd = ScpiCmd::new();
    cmd.set_id(ScpiCommand::SetCssPowerState);
    cmd.set_set(ScpiSet::ScpiSetNormal);
    cmd.sender = 0;
    cmd.size = size_of::<u32>() as u32; // sizeof state
    {
        mhu::SecureMsgLock::new();
        unsafe {
            copy(
                &cmd as *const ScpiCmd,
                SCPI_SHARED_MEM_AP_TO_SCP as *mut ScpiCmd,
                1,
            );
        }
    }

    // Populate the command payload
    unsafe {
        write_volatile(SCPI_CMD_PAYLOAD_AP_TO_SCP as *mut u32, state);
    }
    secure_message_send(size_of::<u32>()); // sizeof state

    // SCP does not reply to this command in order to avoid MHU interrupts
    // from the sender, which could interfere with its power state request.
}

pub fn sys_power_state(system_state: ScpiSystemState) -> ScpiResult {
    let mut response = ScpiCmd::new();
    {
        mhu::SecureMsgLock::new();
        unsafe {
            let cmd = SCPI_SHARED_MEM_AP_TO_SCP as *mut ScpiCmd;
            // Populate the command header
            write_volatile(&mut (*cmd).id, ScpiCommand::SysPowerState as u32);
            write_volatile(&mut (*cmd).set, 0);
            write_volatile(&mut (*cmd).sender, 0);
            write_volatile(&mut (*cmd).size, size_of::<u8>() as u32);

            // Populate the command payload
            write_volatile(SCPI_CMD_PAYLOAD_AP_TO_SCP as *mut u8, system_state as u8);
        }
        secure_message_send(size_of::<u8>());

        // If no response is received, fill in an error status
        if !secure_message_receive(&mut response) {
            return ScpiResult::ETimeout;
        }
    }

    ScpiResult::from_u32(response.status)
}
