use super::{ProcessQ, State};
use crate::{driver::topology, process};
use synctools::mcs::{MCSLock, MCSNode};

pub struct Semaphore {
    lock: MCSLock<isize>,
    procs: ProcessQ,
}

impl Semaphore {
    pub fn new(n: isize) -> Semaphore {
        Semaphore {
            lock: MCSLock::new(n),
            procs: ProcessQ::new(),
        }
    }

    pub fn post(&mut self, node: &mut MCSNode<isize>) {
        // disable interrupts
        let _mask = process::InterMask::new();
        let mut n = self.lock.lock(node);
        *n += 1;

        let readyq = process::get_readyq();
        let tbl = process::get_process_table();

        // global lock
        let mut node2 = MCSNode::new();
        let _lock = process::lock(&mut node2);

        while *n > 0 {
            if let Some(id) = self.procs.deque(tbl) {
                tbl[id as usize].state = State::SemWait;
                readyq.enqueue(id, tbl);
                *n -= 1;
            } else {
                break;
            }
        }
    }

    pub fn wait(&mut self, node: &mut MCSNode<isize>) {
        // disable interrupts
        let mask = process::InterMask::new();

        let mut n = self.lock.lock(node);
        *n -= 1;
        if *n < 0 {
            let actives = process::get_actives();
            let aff = topology::core_pos();
            let tbl = process::get_process_table();

            // global lock
            let mut node2 = MCSNode::new();
            let lock = process::lock(&mut node2);

            // make current thread SemWait
            if let Some(id) = actives[aff] {
                tbl[id as usize].state = State::SemWait;
                self.procs.enqueue(id, tbl);
            }

            actives[aff] = None;

            n.unlock();

            process::schedule2(mask, lock);
        }
    }
}
