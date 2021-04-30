// super visor call (from EL0 to EL1)
pub const SYS_SPAWN: u64 = 1;
pub const SYS_EXIT: u64 = 2;
pub const SYS_SCHED: u64 = 3;
pub const SYS_GETPID: u64 = 4;
pub const SYS_SEND: u64 = 5;
pub const SYS_RECV: u64 = 6;

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

/// Create a new process.
pub fn spawn(app: usize) -> Option<u32> {
    let ret = syscall!(SYS_SPAWN, app);
    if ret < 0 {
        None
    } else {
        Some(ret as u32)
    }
}

/// Quit the process.
pub fn exit() -> ! {
    syscall!(SYS_EXIT);
    unreachable!()
}

/// Yielding.
pub fn sched_yield() {
    syscall!(SYS_SCHED);
}

/// Get the process ID.
pub fn getpid() -> u32 {
    let id = syscall!(SYS_GETPID);
    id as u32
}

/// Send val to dst
pub fn send(dst: u32, val: u32) -> bool {
    if syscall!(SYS_SEND, dst as u64, val as u64) == 1 {
        true
    } else {
        false
    }
}

/// Receive a value
pub fn recv() -> u32 {
    syscall!(SYS_RECV) as u32
}
