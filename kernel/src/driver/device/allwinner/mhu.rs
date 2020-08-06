use super::mbox;

pub fn mhu_secure_message_wait() -> u32 {
    mbox::mhu_secure_message_wait()
}

pub fn mhu_secure_message_send(slot_id: u32) {
    mbox::mhu_secure_message_send(slot_id);
}

pub type SecureMsgLock<'a> = mbox::SecureMsgLock<'a>;
