use crate::{aarch64, driver::int};
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
    handler: fn(T, &DevIRQManger),
}

pub trait IRQHandler {
    fn handle(&mut self) {}
}

pub trait IRQManager {
    type IRQNumberType;

    fn enable(&self, irq_num: Self::IRQNumberType);
    fn disable(&self, irq_num: Self::IRQNumberType);
    fn ack(&self, irq_num: Self::IRQNumberType);
    fn handle(&self, irq_num: Self::IRQNumberType);

    fn register_handler(&mut self, irq_num: Self::IRQNumberType, handler: IRQ<Self::IRQNumberType>);
}

const IRQ_MANAGER: rwlock::RwLock<DevIRQManger> = rwlock::RwLock::new(DevIRQManger::new());
