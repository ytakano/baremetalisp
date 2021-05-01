use crate::driver::topology::core_pos;

use super::*;
use ::alloc::sync::Arc;
use synctools::mcs::{MCSLock, MCSNode};

const QUEUE_SIZE: usize = 8;

struct RingQ<T> {
    buf: [Option<T>; QUEUE_SIZE],
    head: usize,
    last: usize,
    num: usize,
}

pub(super) struct Chan<T> {
    q: MCSLock<RingQ<T>>,
    pid: u8,
}

impl<T: Send> RingQ<T> {
    const BIT_MASK: usize = QUEUE_SIZE - 1;

    fn new() -> Self {
        // check N == 2^x
        assert!((QUEUE_SIZE != 0) && (QUEUE_SIZE & (QUEUE_SIZE - 1)) == 0);

        RingQ {
            buf: arr![None; 8], // QUEUE_SIZE == 8
            head: 0,
            last: 0,
            num: 0,
        }
    }

    fn enque(&mut self, v: T) -> Result<(), T> {
        if self.num >= QUEUE_SIZE {
            return Err(v);
        }

        self.buf[self.last] = Some(v);
        self.num += 1;
        self.last += 1;

        Ok(())
    }

    fn deque(&mut self) -> Option<T> {
        if self.num == 0 {
            return None;
        }

        let v = self.buf[self.head].take();
        self.head += 1;
        self.num -= 1;

        v
    }
}

impl<T: Send> Chan<T> {
    pub(super) fn new(pid: u8) -> Self {
        Chan {
            q: MCSLock::new(RingQ::new()),
            pid,
        }
    }

    pub(super) fn set_pid(&mut self, pid: u8) {
        self.pid = pid;
    }

    pub(super) fn channel(self) -> (Sender<T>, Receiver<T>) {
        let ch = Arc::new(self);
        (Sender { ch: ch.clone() }, Receiver { ch })
    }
}

#[derive(Clone)]
pub(super) struct Sender<T> {
    ch: Arc<Chan<T>>,
}

impl<T: Send> Sender<T> {
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

        q.unlock();
        schedule2(mask, proc_info);

        Ok(())
    }

    pub(super) fn into_raw(self) -> *const Chan<T> {
        Arc::into_raw(self.ch)
    }

    pub(super) unsafe fn from_raw(ptr: *const Chan<T>) -> Self {
        Sender {
            ch: Arc::from_raw(ptr),
        }
    }
}

pub(super) struct Receiver<T> {
    ch: Arc<Chan<T>>,
}

impl<T: Send> Receiver<T> {
    pub(super) fn recv(&self) -> T {
        let mut node = MCSNode::new();

        loop {
            let mask = InterMask::new();
            let mut q = self.ch.q.lock(&mut node);
            if let Some(r) = q.deque() {
                return r;
            } else {
                let aff = core_pos();
                let actives = get_actives();
                let current = actives[aff].unwrap(); // must be active

                // make this thread's state Recv
                let mut node = MCSNode::new();
                let mut proc_info = PROC_INFO.lock(&mut node);

                if let Some(entry) = proc_info.table[current as usize].as_mut() {
                    entry.state = State::Recv;
                } else {
                    panic!("no current process");
                }

                q.unlock();
                schedule2(mask, proc_info);
            }
        }
    }

    pub(super) fn into_raw(self) -> *const Chan<T> {
        Arc::into_raw(self.ch)
    }

    pub(super) unsafe fn from_raw(ptr: *const Chan<T>) -> Self {
        Receiver {
            ch: Arc::from_raw(ptr),
        }
    }
}
