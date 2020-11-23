use super::mbox;

pub(in crate::driver) fn mhu_secure_message_wait() -> u32 {
    mbox::mhu_secure_message_wait()
}

pub(in crate::driver) fn mhu_secure_message_send(slot_id: u32) {
    mbox::mhu_secure_message_send(slot_id);
}

pub(in crate::driver) type SecureMsgLock<'a> = mbox::SecureMsgLock<'a>;
