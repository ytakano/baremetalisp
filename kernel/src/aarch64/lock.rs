use core::intrinsics::volatile_load;

pub struct LockVar {
    var: u64,
}

impl LockVar {
    pub const fn new() -> LockVar {
        LockVar { var: 0 }
    }

    pub fn lock(&mut self) -> SpinLock {
        SpinLock::new(&mut self.var)
    }
}

pub struct SpinLock<'a> {
    lock: &'a mut u64,
}

impl<'a> SpinLock<'a> {
    fn new(n: &'a mut u64) -> SpinLock<'a> {
        loop {
            if 0 == unsafe { volatile_load(n) } {
                if test_and_set(n) {
                    return SpinLock { lock: n };
                }
            }
        }
    }
}

impl<'a> Drop for SpinLock<'a> {
    fn drop(&mut self) {
        *self.lock = 0;
    }
}

fn test_and_set(n: &mut u64) -> bool {
    let mut rd: u64;
    let addr = n as *mut u64 as u64;
    unsafe {
        asm! (
            "mov   {2}, #1
             1:
             ldaxr {3}, [{0}]
             stlxr {4:w}, {2}, [{0}]
             cbnz  {4:w}, 1b
             clrex
             and   {1}, {3}, #1",
            in(reg) addr,
            lateout(reg) rd,
            out(reg) _,
            out(reg) _,
            out(reg) _,
        );
    }
    rd == 0
}
