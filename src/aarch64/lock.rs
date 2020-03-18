use core::intrinsics::volatile_load;

pub struct LockVar {
    var: u64
}

impl LockVar {
    pub fn new() -> LockVar {
        LockVar{var: 0}
    }

    pub fn lock(&mut self) -> SpinLock {
        SpinLock::new(&mut self.var)
    }
}

pub struct SpinLock<'a> {
    lock: &'a mut u64
}

impl<'a> SpinLock<'a> {
    pub fn new(n: &'a mut u64) -> SpinLock<'a> {
        loop {
            if 0 == unsafe { volatile_load(n) } {
                if test_and_set(n) {
                    return SpinLock{lock: n};
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
            "mov   x8, #1;
             1:
             ldaxr x9, [$1];
             stlxr w10, x8, [$1];
             cbnz  w10, 1b;
             clrex;
             and   $0, x9, #1"
            : "=r"(rd)
            : "r"(addr)
            : "x8", "x9", "w10"
        );
    }
    rd == 0
}