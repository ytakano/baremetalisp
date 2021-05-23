use crate::{
    bsp::int::{self, IRQ},
    global::GlobalVar,
    mmio::ReadWrite,
    out,
};
use arr_macro::arr;
use core::default::Default;
use synctools::mcs::{MCSLock, MCSNode};

const GIC_MAX_INTS: usize = 1020;
const NUM_INTS_PER_REG: usize = 32;
const NUM_TARGETS_PER_REG: usize = 4;
const NUM_SGI: usize = 16;

const ITARGETSR_FIELD_BITS: usize = 8;
const ITARGETSR_FIELD_MASK: usize = 0xff;

const GICC_CTLR_ENABLEGRP0: u32 = 1 << 0;
const GICC_CTLR_ENABLEGRP1: u32 = 1 << 1;
const GICD_CTLR_ENABLEGRP1S: u32 = 1 << 2;
const GICC_CTLR_FIQEN: u32 = 1 << 3;

const GICD_CTLR_ENABLEGRP0: u32 = 1 << 0;
const GICD_CTLR_ENABLEGRP1: u32 = 1 << 1;

static GIC_GLOBAL: MCSLock<GlobalVar<GIC>> = MCSLock::new(GlobalVar::UnInit);

#[derive(Default)]
pub struct GIC {
    gicc_base: usize,
    gicd_base: usize,
    ver: GICVer,
    max_it: usize,
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum GICVer {
    V2,
    V3,
}

impl Default for GICVer {
    fn default() -> Self {
        GICVer::V2
    }
}

pub fn init(gicc_base: usize, gicd_base: usize, ver: GICVer) {
    let mut node = MCSNode::new();
    let mut lock = GIC_GLOBAL.lock(&mut node);

    let mut g = GIC {
        gicc_base,
        gicd_base,
        ver,
        max_it: 0,
    };

    // calculate the maximum number of interrupt
    g.probe_max_it();

    for n in 0..=(g.max_it / NUM_INTS_PER_REG) {
        let icen_ptr = g.gicd_icenabler(n);
        let icpend_ptr = g.gicd_icpendr(n);
        let igroupr_ptr = g.gicd_igroupr(n);

        // Disable interrupts
        icen_ptr.write(0xffffffff);

        // Make interrupts non-pending
        icpend_ptr.write(0xffffffff);

        // Mark interrupts non-secure
        if n == 0 {
            // per-CPU inerrupts config:
            // ID0-ID7(SGI)   for Non-secure interrupts
            // ID8-ID15(SGI)  for Secure interrupts.
            // All PPI config as Non-secure interrupts.
            igroupr_ptr.write(0xffff00ff);
        } else {
            igroupr_ptr.write(0xffffffff);
        }
    }

    g.gicc_pmr().write(0x80);

    // Enable GIC
    let gicc_ctlr = g.gicc_ctlr();
    let gicd_ctlr = g.gicd_ctlr();
    gicc_ctlr.write(GICC_CTLR_FIQEN | GICC_CTLR_ENABLEGRP0 | GICC_CTLR_ENABLEGRP1);
    gicd_ctlr.setbits(GICD_CTLR_ENABLEGRP0 | GICD_CTLR_ENABLEGRP1);

    out::msg("GICv2", "Initialized");

    if let GlobalVar::UnInit = *lock {
        *lock = GlobalVar::Having(g)
    } else {
        panic!("initialized twice");
    }
}

pub fn take() -> Option<GIC> {
    let mut node = MCSNode::new();
    let mut lock = GIC_GLOBAL.lock(&mut node);
    let gic = lock.take();
    lock.unlock();

    if let GlobalVar::Having(g) = gic {
        Some(g)
    } else {
        None
    }
}

impl Drop for GIC {
    fn drop(&mut self) {
        let mut node = MCSNode::new();
        let mut lock = GIC_GLOBAL.lock(&mut node);
        *lock = GlobalVar::Having(GIC {
            gicc_base: self.gicc_base,
            gicd_base: self.gicd_base,
            ver: self.ver,
            max_it: self.max_it,
        });
    }
}

impl GIC {
    fn gicc_ctlr(&self) -> ReadWrite<u32> {
        ReadWrite::new(self.gicc_base)
    }

    fn gicc_pmr(&self) -> ReadWrite<u32> {
        ReadWrite::new(self.gicc_base + 0x04)
    }

    fn gicd_ctlr(&self) -> ReadWrite<u32> {
        ReadWrite::new(self.gicd_base)
    }

    fn gicd_isenabler(&self, n: usize) -> ReadWrite<u32> {
        ReadWrite::new(self.gicd_base + 0x100 + n * 4)
    }

    fn gicd_icenabler(&self, n: usize) -> ReadWrite<u32> {
        ReadWrite::new(self.gicd_base + 0x180 + n * 4)
    }

    fn gicd_ispendr(&self, n: usize) -> ReadWrite<u32> {
        ReadWrite::new(self.gicd_base + 0x200 + n * 4)
    }

    fn gicd_icpendr(&self, n: usize) -> ReadWrite<u32> {
        ReadWrite::new(self.gicd_base + 0x280 + n * 4)
    }

    fn gicd_igroupr(&self, n: usize) -> ReadWrite<u32> {
        ReadWrite::new(self.gicd_base + 0x80 + n * 4)
    }

    fn gicd_itargetsr(&self, n: usize) -> ReadWrite<u32> {
        ReadWrite::new(self.gicd_base + 0x800 + n * 4)
    }

    fn probe_max_it(&mut self) {
        let max_regs = ((GIC_MAX_INTS + NUM_INTS_PER_REG - 1) >> 5) - 1;
        let gicc_ctlr = self.gicc_ctlr();

        let old_ctlr = gicc_ctlr.read();
        gicc_ctlr.write(0);

        'out: for i in (0..=max_regs).rev() {
            let set = self.gicd_isenabler(i);
            let cancel = self.gicd_icenabler(i);

            let old_reg = set.read();
            set.write(0xffffffff);
            let reg = set.read();
            cancel.write(!old_reg);
            for b in (0..NUM_INTS_PER_REG).rev() {
                if (1 << b) & reg > 0 {
                    self.max_it = i * NUM_INTS_PER_REG + b;
                    break 'out;
                }
            }
        }

        gicc_ctlr.write(old_ctlr);
    }

    fn it_add(&self, it: usize) {
        if it > self.max_it {
            return;
        }

        let idx = it >> 5;
        let mask = 1 << (it & (NUM_INTS_PER_REG - 1)) as u32;

        self.gicd_icenabler(idx).write(mask); // disable
        self.gicd_icpendr(idx).write(mask); // make it non-pending
        self.gicd_igroupr(idx).clrbits(mask); // assign to group0
    }

    fn it_set_cpu_mask(&self, it: usize, cpu_mask: u8) {
        let itargetsr = self.gicd_itargetsr(it >> 2);

        let mut target = itargetsr.read();
        let target_shift = (it & (NUM_TARGETS_PER_REG - 1)) * ITARGETSR_FIELD_BITS;
        target &= !(ITARGETSR_FIELD_MASK << target_shift) as u32;
        target |= (cpu_mask as u32) << target_shift;
        itargetsr.write(target);
    }

    fn it_enable(&self, it: usize) -> bool {
        if it > self.max_it {
            return false;
        }

        let idx = it >> 5;
        let mask = 1 << (it & (NUM_INTS_PER_REG - 1)) as u32;
        let isen = self.gicd_isenabler(idx);

        if self.gicd_igroupr(idx).read() & mask != 0 {
            return false;
        }

        if it >= NUM_SGI {
            if isen.read() & mask != 0 {
                return false;
            }
        }

        isen.write(mask);
        true
    }
}

pub type IRQNumber = u16;

pub struct IRQManager {
    handlers: [Option<IRQ<IRQNumber>>; GIC_MAX_INTS],
}

impl int::IRQManager for IRQManager {
    type IRQNumberType = IRQNumber;

    fn enable(&self, _irq_num: Self::IRQNumberType) {}
    fn disable(&self, _irq_num: Self::IRQNumberType) {}
    fn ack(&self, _irq_num: Self::IRQNumberType) {}
    fn handle(&self, _irq_num: Self::IRQNumberType) {}

    fn new() -> Self {
        IRQManager {
            handlers: arr![None; 1020],
        }
    }

    fn register_handler(
        &mut self,
        irq_num: Self::IRQNumberType,
        handler: IRQ<Self::IRQNumberType>,
    ) {
        self.handlers[irq_num as usize] = Some(handler);
    }
}
