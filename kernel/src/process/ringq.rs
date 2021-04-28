use crate::driver::topology::core_pos;

use super::*;
use ::alloc::sync::Arc;
use core::mem::MaybeUninit;
use synctools::mcs::{MCSLock, MCSNode};

struct RingQ<T, const N: usize> {
    buf: [Option<T>; N],
    head: usize,
    last: usize,
}

pub(super) struct Chan<T, const N: usize> {
    q: MCSLock<RingQ<T, N>>,
    pid: u8,
}

impl<T: Send, const N: usize> RingQ<T, N> {
    const BIT_MASK: usize = N - 1;

    fn new() -> Self {
        crate::driver::uart::puts("RingQ::new() 0\n");

        // check N == 2^x
        assert!((N != 0) && (N & (N - 1)) == 0);
        crate::driver::uart::puts("RingQ::new() 1\n");

        RingQ {
            buf: unsafe { MaybeUninit::uninit().assume_init() },
            head: 0,
            last: 0,
        }
    }

    fn enque(&mut self, v: T) -> Result<(), T> {
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

impl<T: Send, const N: usize> Chan<T, N> {
    pub(super) fn new(pid: u8) -> (Sender<T, N>, Receiver<T, N>) {
        crate::driver::uart::puts("Chan::new() 0\n");

        let ch = Chan {
            q: MCSLock::new(RingQ::new()),
            pid,
        };

        crate::driver::uart::puts("Chan::new() 1\n");

        let ch = Arc::new(ch);

        (Sender { ch: ch.clone() }, Receiver { ch })
    }
}

#[derive(Clone)]
pub(super) struct Sender<T, const N: usize> {
    ch: Arc<Chan<T, N>>,
}

impl<T: Send, const N: usize> Sender<T, N> {
    pub(super) fn send(&self, v: T) -> Result<(), T> {
        let mut node = MCSNode::new();
        let mask = InterMask::new();
        let mut q = self.ch.q.lock(&mut node);
        let _ = q.enque(v)?;

        // notify to the receiver
        let mut node = MCSNode::new();
        let mut proc_info = PROC_INFO.lock(&mut node);
        let (tbl, readyq) = proc_info.get_mut();
        if let Some(entry) = tbl[self.ch.pid as usize].as_mut() {
            if entry.state == State::Recv {
                entry.state = State::Ready;
                readyq.enque(self.ch.pid, tbl);
            }
        }

        proc_info.unlock();
        q.unlock();
        mask.unmask();

        schedule();

        Ok(())
    }
}

pub(super) struct Receiver<T, const N: usize> {
    ch: Arc<Chan<T, N>>,
}

impl<T: Send, const N: usize> Receiver<T, N> {
    pub(super) fn recv(&self) -> T {
        let mut node = MCSNode::new();

        loop {
            let mask = InterMask::new();
            let mut q = self.ch.q.lock(&mut node);
            if let Some(r) = q.deque() {
                return r;
            } else {
                {
                    // make this thread's state Recv
                    let aff = core_pos();
                    let actives = get_actives();
                    let current = actives[aff].unwrap(); // must be active

                    let mut node = MCSNode::new();
                    let mut proc_info = PROC_INFO.lock(&mut node);

                    if let Some(entry) = proc_info.table[current as usize].as_mut() {
                        entry.state = State::Recv;
                    } else {
                        panic!("no current process");
                    }
                    actives[aff] = None;
                }

                q.unlock();
                mask.unmask();

                schedule();
            }
        }
    }
}
