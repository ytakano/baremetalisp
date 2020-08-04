use core::intrinsics::copy;

use crate::driver::memory::CSS_SCP_COM_SHARED_MEM_BASE;
use crate::driver::mhu;

const SCPI_SHARED_MEM_SCP_TO_AP: u32 = CSS_SCP_COM_SHARED_MEM_BASE;
const SCPI_SHARED_MEM_AP_TO_SCP: u32 = CSS_SCP_COM_SHARED_MEM_BASE + 0x100;

const SCPI_MHU_SLOT_ID: u32 = 0;

enum ScpiSet {
    ScpiSetNormal = 0, // Normal SCPI commands
    ScpiSetExtended,   // Extended SCPI commands
}

#[derive(PartialEq, Clone, Copy)]
enum ScpiResult {
    ScpOk = 0,    // Success
    ScpEParam,    // Invalid parameter(s)
    ScpEAlign,    // Invalid alignment
    ScpESize,     // Invalid size
    ScpEHandler,  // Invalid handler or callback
    ScpEAccess,   // Invalid access or permission denied
    ScpERange,    // Value out of range
    ScpETimeout,  // Time out has ocurred
    ScpENomem,    // Invalid memory area or pointer
    ScpEPwrstate, // Invalid power state
    ScpESupport,  // Feature not supported or disabled
    ScpiEDevice,  // Device error
    ScpiEBusy,    // Device is busy
}

enum ScpiCommand {
    ScpiCmdScpReady = 0x01,
    ScpiCmdSetCssPowerState = 0x03,
    ScpiCmdGetCssPowerState = 0x04,
    ScpiCmdSysPowerStat = 0x00,
}

enum ScpiPowerState {
    ScpiPowerOn = 0,
    ScpiPowerRetention = 1,
    ScpiPowerOff = 3,
}

enum ScpiSystemState {
    ScpiSystemShutdown = 0,
    ScpiSystemReboot = 1,
    ScpiSystemReset = 2,
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
}

pub fn scpi_wait_ready() -> bool {
    let mut cmd = ScpiCmd::new();

    {
        // Get a message from the SCP
        mhu::SecureMsgLock::new();
        if !scpi_secure_message_receive(&mut cmd) {
            // If no message was received, don't send a response
            return false;
        }
    }

    // We are expecting 'SCP Ready', produce correct error if it's not
    let status = if cmd.id != ScpiCommand::ScpiCmdScpReady as u32 {
        ScpiResult::ScpESupport
    } else if cmd.size != 0 {
        ScpiResult::ScpESize
    } else {
        ScpiResult::ScpOk
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

        scpi_secure_message_send(0);
    }

    status == ScpiResult::ScpOk
}

pub fn scpi_secure_message_send(_payload_size: usize) {
    // Ensure that any write to the SCPI payload area is seen by SCP before
    // we write to the MHU register. If these 2 writes were reordered by
    // the CPU then SCP would read stale payload data
    unsafe {
        asm!("dmb st");
    }

    mhu::mhu_secure_message_send(SCPI_MHU_SLOT_ID);
}

pub fn scpi_secure_message_receive(cmd: &mut ScpiCmd) -> bool {
    let mhu_status = mhu::mhu_secure_message_wait();

    // Expect an SCPI message, reject any other protocol
    if mhu_status != (1 << SCPI_MHU_SLOT_ID) {
        return false;
    }

    unsafe {
        // Ensure that any read to the SCPI payload area is done after reading
        // the MHU register. If these 2 reads were reordered then the CPU would
        // read invalid payload data
        asm!("dmb ld");

        copy(
            SCPI_SHARED_MEM_SCP_TO_AP as *const ScpiCmd,
            cmd as *mut ScpiCmd,
            1,
        );
    }

    true
}
