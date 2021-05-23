/// Board specific interrupt handler
use crate::global::GlobalVar;
use synctools::rwlock;

// Raspberry Pi 4, Broadcom BCM2xxx
#[cfg(any(feature = "raspi3", feature = "raspi4"))]
type DevIRQManager = super::raspi::int::IRQManager;

// Raspberry Pi 4, Broadcom BCM2xxx
#[cfg(any(feature = "raspi3", feature = "raspi4"))]
pub type DevIRQNumber = super::raspi::int::IRQNumber;

// Pine64, Allwineer sunxi
#[cfg(feature = "pine64")]
type DevIRQManager = crate::driver::gic::IRQManager;

// Pine64, Allwineer sunxi
#[cfg(feature = "pine64")]
pub type DevIRQNumber = crate::driver::gic::IRQNumber;

impl DevIRQManager where DevIRQManager: IRQManager {}

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

static IRQ_MANAGER: rwlock::RwLock<GlobalVar<DevIRQManager>> =
    rwlock::RwLock::new(GlobalVar::UnInit);

pub fn init() {
    let mut lock = IRQ_MANAGER.write();
    if let GlobalVar::UnInit = *lock {
        *lock = GlobalVar::Having(DevIRQManager::new());
    } else {
        panic!("initialized twice");
    }
}

pub fn enable_irq_num(irq_num: DevIRQNumber) {
    let lock = IRQ_MANAGER.read();
    if let GlobalVar::Having(mng) = &*lock {
        mng.enable(irq_num);
    }
}

pub fn disable_irq_num(irq_num: DevIRQNumber) {
    let lock = IRQ_MANAGER.read();
    if let GlobalVar::Having(mng) = &*lock {
        mng.disable(irq_num);
    }
}

pub fn register_handler(irq_num: DevIRQNumber, handler: IRQ<DevIRQNumber>) {
    let mut lock = IRQ_MANAGER.write();
    if let GlobalVar::Having(mng) = &mut *lock {
        mng.register_handler(irq_num, handler);
    }
}
