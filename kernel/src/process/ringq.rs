use core::mem::MaybeUninit;
use synctools::mcs::MCSLock;

struct RingQ<T, const N: usize> {
    buf: [Option<T>; N],
    head: usize,
    last: usize,
}

pub struct ProcToDev<T, const N: usize> {
    q: MCSLock<RingQ<T, N>>,
}

pub struct DevToProc<T, const N: usize> {
    q: MCSLock<RingQ<T, N>>,
}

impl<T: Send, const N: usize> RingQ<T, N> {
    const BIT_MASK: usize = N - 1;

    fn new() -> RingQ<T, N> {
        // check N == 2^x
        assert!((N != 0) && (N & (N - 1)) == 0);

        RingQ {
            buf: unsafe { MaybeUninit::uninit().assume_init() },
            head: 0,
            last: 0,
        }
    }

    fn enqueue(&mut self, v: T) -> Result<(), T> {
        let next = (self.last + 1) & Self::BIT_MASK;
        if next == self.head {
            Err(v)
        } else {
            self.last = next;
            self.buf[next] = Some(v);
            Ok(())
        }
    }

    fn deque(&mut self) -> Option<T> {
        if self.head == self.last {
            None
        } else {
            let v = self.buf[self.head].take();
            self.buf[self.head] = None;
            self.head += 1;
            Some(v.unwrap())
        }
    }
}
