use crate::{aarch64, driver::int, global::GlobalVar};
use synctools::rwlock;

pub trait InterMask {
    fn new() -> Self;
    fn enable_irq();
    fn unmask(self);
}

pub type ArchIntMask = aarch64::int::AA64Mask;

impl ArchIntMask where ArchIntMask: InterMask {}

pub fn mask() -> ArchIntMask {
    ArchIntMask::new()
}

pub fn enable_irq() {
    ArchIntMask::enable_irq();
}

type DevIRQManger = int::DevIRQManger;

impl DevIRQManger where DevIRQManger: IRQManager {}

pub struct IRQ<T> {
    description: &'static str,
    handler: fn(T),
}

impl<T> IRQ<T> {
    pub fn handle(&self, n: T) {
        (self.handler)(n);
    }
}

pub trait IRQHandler {
    fn handle(&mut self) {}
}

pub trait IRQManager {
    type IRQNumberType;

    fn new() -> Self;
    fn enable(&self, irq_num: Self::IRQNumberType);
    fn disable(&self, irq_num: Self::IRQNumberType);
    fn ack(&self, irq_num: Self::IRQNumberType);
    fn handle(&self, irq_num: Self::IRQNumberType);

    fn register_handler(&mut self, irq_num: Self::IRQNumberType, handler: IRQ<Self::IRQNumberType>);
}

static IRQ_MANAGER: rwlock::RwLock<GlobalVar<DevIRQManger>> =
    rwlock::RwLock::new(GlobalVar::UnInit);

pub fn init() {
    let mut lock = IRQ_MANAGER.write();
    if let GlobalVar::UnInit = *lock {
        *lock = GlobalVar::Having(DevIRQManger::new());
    } else {
        panic!("initialized twice");
    }
}
