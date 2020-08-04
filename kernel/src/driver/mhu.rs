#[cfg(feature = "pine64")]
use super::device::a64::mhu;

pub type SecureMsgLock<'a> = mhu::SecureMsgLock<'a>;

pub fn mhu_secure_message_wait() -> u32 {
    mhu::mhu_secure_message_wait()
}

pub fn mhu_secure_message_send(slot_id: u32) {
    mhu::mhu_secure_message_send(slot_id);
}
