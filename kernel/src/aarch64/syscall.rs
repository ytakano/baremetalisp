use crate::{
    process,
    syscall::{self, Locator},
};

use super::context::GpRegs;

pub(super) fn handle64(regs: &GpRegs) -> i64 {
    match regs.x0 {
        syscall::SYS_SPAWN => {
            if let Some(pid) = process::spawn(regs.x1) {
                pid as i64
            } else {
                -1
            }
        }
        syscall::SYS_EXIT => process::exit(),
        syscall::SYS_SCHED => {
            process::schedule();
            0
        }
        syscall::SYS_GETPID => process::get_id() as i64,
        syscall::SYS_RECV => {
            let src = unsafe { &mut *(regs.x1 as *mut Locator) };
            process::recv(src) as i64
        }
        syscall::SYS_SEND => {
            let dst = unsafe { &*(regs.x1 as *const Locator) };
            if process::send(dst, regs.x2 as u32) {
                1
            } else {
                0
            }
        }
        _ => 0,
    }
}
