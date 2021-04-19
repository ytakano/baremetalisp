use crate::thread;

use super::context::GpRegs;

// super visor call (from EL0 to EL1)
pub const SYS_SPAWN: u64 = 1;
pub const SYS_EXIT: u64 = 2;

macro_rules! syscall {
    ($id:expr) => {
        {
            let ret: isize;
            unsafe {
                asm!(
                    "mov x0, {}
                     svc #0
                     mov {}, x0",
                    in(reg) $id,
                    lateout(reg) ret
                )
            };
            ret
        }
    };
    ($id:expr, $arg1:expr) => {
        {
            let ret: isize;
            unsafe {
                asm!(
                    "mov x0, {}
                     mov x1, {}
                     svc #0
                     mov {}, x0",
                    in(reg) $id,
                    in(reg) $arg1,
                    lateout(reg) ret
                )
            };
            ret
        }
    };
    ($id:expr, $arg1:expr, $arg2:expr) => {
        {
            let ret: isize;
            unsafe {
                asm!(
                    "mov x0, {}
                     mov x1, {}
                     mov x2, {}
                     svc #0
                     mov {}, x0",
                    in(reg) $id,
                    in(reg) $arg1,
                    in(reg) $arg2,
                    lateout(reg) ret
                )
            };
            ret
        }
    };
}

pub fn spawn(app: usize) -> Option<u8> {
    let ret = syscall!(SYS_SPAWN, app);
    if ret < 0 {
        None
    } else {
        Some(ret as u8)
    }
}

pub fn exit() {
    syscall!(SYS_EXIT);
}

pub(super) fn handle64(id: u64, arg1: u64, _arg2: u64, regs: &GpRegs) -> i64 {
    match id {
        SYS_SPAWN => {
            thread::spawn(arg1, Some(regs));
            unreachable!()
        }
        SYS_EXIT => -1,
        _ => 0,
    }
}
