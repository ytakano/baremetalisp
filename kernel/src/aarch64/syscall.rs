use crate::{syscall, thread};

use super::context::GpRegs;

pub(super) fn handle64(id: u64, arg1: u64, _arg2: u64, regs: &GpRegs) -> i64 {
    match id {
        syscall::SYS_SPAWN => {
            thread::spawn(arg1, Some(regs));
            -1
        }
        syscall::SYS_EXIT => thread::exit(),
        syscall::SYS_SCHED => thread::schedule(),
        syscall::SYS_GETPID => thread::get_id() as i64,
        _ => 0,
    }
}
