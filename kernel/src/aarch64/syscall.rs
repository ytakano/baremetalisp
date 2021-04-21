use crate::{process, syscall};

use super::context::GpRegs;

pub(super) fn handle64(id: u64, arg1: u64, _arg2: u64, regs: &GpRegs) -> i64 {
    match id {
        syscall::SYS_SPAWN => {
            process::spawn(arg1, Some(regs));
            -1
        }
        syscall::SYS_EXIT => process::exit(),
        syscall::SYS_SCHED => process::schedule(),
        syscall::SYS_GETPID => process::get_id() as i64,
        _ => 0,
    }
}
