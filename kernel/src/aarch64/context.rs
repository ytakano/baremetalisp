use crate::driver::topology::CORE_COUNT;
use crate::psci::ep_info::EntryPointInfo;

static mut CPU_CONTEXT_SECURE: [CPUContext; CORE_COUNT] = [CPUContext::new(); CORE_COUNT];
static mut CPU_CONTEXT_NON_SECURE: [CPUContext; CORE_COUNT] = [CPUContext::new(); CORE_COUNT];

/// General Purpose Registers
#[derive(Copy, Clone)]
#[repr(C)]
pub struct GpRegs {
    pub x0: u64,
    pub x1: u64,
    pub x2: u64,
    pub x3: u64,
    pub x4: u64,
    pub x5: u64,
    pub x6: u64,
    pub x7: u64,
    pub x8: u64,
    pub x9: u64,
    pub x10: u64,
    pub x11: u64,
    pub x12: u64,
    pub x13: u64,
    pub x14: u64,
    pub x15: u64,
    pub x16: u64,
    pub x17: u64,
    pub x18: u64,
    pub x19: u64,
    pub x20: u64,
    pub x21: u64,
    pub x22: u64,
    pub x23: u64,
    pub x24: u64,
    pub x25: u64,
    pub x26: u64,
    pub x27: u64,
    pub x28: u64,
    pub x29: u64,
    pub x30: u64,  // link register
    pub elr: u64,  // exception link register
    pub spsr: u32, // saved program status register
    _unused: [u8; 12],
}

impl GpRegs {
    pub const fn new() -> GpRegs {
        GpRegs {
            x0: 0,
            x1: 0,
            x2: 0,
            x3: 0,
            x4: 0,
            x5: 0,
            x6: 0,
            x7: 0,
            x8: 0,
            x9: 0,
            x10: 0,
            x11: 0,
            x12: 0,
            x13: 0,
            x14: 0,
            x15: 0,
            x16: 0,
            x17: 0,
            x18: 0,
            x19: 0,
            x20: 0,
            x21: 0,
            x22: 0,
            x23: 0,
            x24: 0,
            x25: 0,
            x26: 0,
            x27: 0,
            x28: 0,
            x29: 0,
            x30: 0,
            elr: 0,
            spsr: 0,
            _unused: [0; 12],
        }
    }
}

/// System Registers of EL1 and EL0
#[derive(Copy, Clone)]
pub struct EL1SysRegs {
    pub sctlr_el1: u64,
    pub actlr_el1: u64,
    pub cpacr_el1: u64,
    pub csselr_el1: u64,
    pub sp_el1: u64,
    pub esr_el1: u64,
    pub ttbr0_el1: u64,
    pub ttbr1_el1: u64,
    pub mair_el1: u64,
    pub amair_el1: u64,
    pub tcr_el1: u64,
    pub tpidr_el1: u64,
    pub tpidr_el0: u64,
    pub tpidrro_el0: u64,
    pub par_el1: u64,
    pub far_el1: u64,
    pub afsr0_el1: u64,
    pub afsr1_el1: u64,
    pub contextidr_el1: u64,
    pub vbar_el1: u64,
    // If the platform is AArch64-only, there is no need to save and restore these
    // AArch32 registers.
    // spsr_abt: u64,
    // spsr_udt: u64,
    // spsr_irq: u64,
    // spsr_fiq: u64,
    // dacr32_el2: u64,
    // ifsr32_el2: u64,
    // If the timer registers aren't saved and restored, we don't have to reserve
    // space for them in the context
    pub cntp_ctl_el0: u64,
    pub cntp_cval_el0: u64,
    pub cntv_ctl_el0: u64,
    pub cntv_cval_el0: u64,
    pub cntkctl_el1: u64,
    // MTE regs (from Armv 8.5)
    // tfsr0_el1: u64,
    // tfsr_el1: u64,
    // rgsr_el1: u64,
    // gcr_el1: u64,
}

impl EL1SysRegs {
    pub const fn new() -> EL1SysRegs {
        EL1SysRegs {
            sctlr_el1: 0,
            actlr_el1: 0,
            cpacr_el1: 0,
            csselr_el1: 0,
            sp_el1: 0,
            esr_el1: 0,
            ttbr0_el1: 0,
            ttbr1_el1: 0,
            mair_el1: 0,
            amair_el1: 0,
            tcr_el1: 0,
            tpidr_el1: 0,
            tpidr_el0: 0,
            tpidrro_el0: 0,
            par_el1: 0,
            far_el1: 0,
            afsr0_el1: 0,
            afsr1_el1: 0,
            contextidr_el1: 0,
            vbar_el1: 0,
            cntp_ctl_el0: 0,
            cntp_cval_el0: 0,
            cntv_ctl_el0: 0,
            cntv_cval_el0: 0,
            cntkctl_el1: 0,
        }
    }
}

/// Floating Point Registers
#[derive(Copy, Clone)]
pub struct FPRegs {
    fp_q0: [u8; 16],
    fp_q1: [u8; 16],
    fp_q2: [u8; 16],
    fp_q3: [u8; 16],
    fp_q4: [u8; 16],
    fp_q5: [u8; 16],
    fp_q6: [u8; 16],
    fp_q7: [u8; 16],
    fp_q8: [u8; 16],
    fp_q9: [u8; 16],
    fp_q10: [u8; 16],
    fp_q11: [u8; 16],
    fp_q12: [u8; 16],
    fp_q13: [u8; 16],
    fp_q14: [u8; 16],
    fp_q15: [u8; 16],
    fp_q16: [u8; 16],
    fp_q17: [u8; 16],
    fp_q18: [u8; 16],
    fp_q19: [u8; 16],
    fp_q20: [u8; 16],
    fp_q21: [u8; 16],
    fp_q22: [u8; 16],
    fp_q23: [u8; 16],
    fp_q24: [u8; 16],
    fp_q25: [u8; 16],
    fp_q26: [u8; 16],
    fp_q27: [u8; 16],
    fp_q28: [u8; 16],
    fp_q29: [u8; 16],
    fp_q30: [u8; 16],
    fp_q31: [u8; 16],
    // for AArch32
    // fpexc32_el2: [u8; 16],
    fp_fpsr: u64,
    fp_fpcr: u64,
}

impl FPRegs {
    pub const fn new() -> FPRegs {
        FPRegs {
            fp_q0: [0; 16],
            fp_q1: [0; 16],
            fp_q2: [0; 16],
            fp_q3: [0; 16],
            fp_q4: [0; 16],
            fp_q5: [0; 16],
            fp_q6: [0; 16],
            fp_q7: [0; 16],
            fp_q8: [0; 16],
            fp_q9: [0; 16],
            fp_q10: [0; 16],
            fp_q11: [0; 16],
            fp_q12: [0; 16],
            fp_q13: [0; 16],
            fp_q14: [0; 16],
            fp_q15: [0; 16],
            fp_q16: [0; 16],
            fp_q17: [0; 16],
            fp_q18: [0; 16],
            fp_q19: [0; 16],
            fp_q20: [0; 16],
            fp_q21: [0; 16],
            fp_q22: [0; 16],
            fp_q23: [0; 16],
            fp_q24: [0; 16],
            fp_q25: [0; 16],
            fp_q26: [0; 16],
            fp_q27: [0; 16],
            fp_q28: [0; 16],
            fp_q29: [0; 16],
            fp_q30: [0; 16],
            fp_q31: [0; 16],
            fp_fpsr: 0,
            fp_fpcr: 0,
        }
    }
}

#[derive(Copy, Clone)]
pub struct EL3State {
    pub scr_el3: u64,
    pub esr_el3: u64,
    pub runtime_sp: u64,
    pub spsr_el3: u64,
    pub elr_el3: u64,
    pub pmcr_el3: u64,
}

impl EL3State {
    pub const fn new() -> EL3State {
        EL3State {
            scr_el3: 0,
            esr_el3: 0,
            runtime_sp: 0,
            spsr_el3: 0,
            elr_el3: 0,
            pmcr_el3: 0,
        }
    }
}

/// Top-level context structure which is used by EL3 firmware to
/// preserve the state of a core at EL1 in one of the two security
/// states and save enough EL3 meta data to be able to return to that
/// EL and security state. The context management library will be used
/// to ensure that SP_EL3 always points to an instance of this
/// structure at exception entry and exit. Each instance will
/// correspond to either the secure or the non-secure state.
#[derive(Copy, Clone)]
pub struct CPUContext {
    gpregx_ctx: GpRegs,
    el3state_ctx: EL3State,
    el1_sysregs_ctx: EL1SysRegs,
    fpregs_ctx: FPRegs,
}

impl CPUContext {
    pub const fn new() -> CPUContext {
        CPUContext {
            gpregx_ctx: GpRegs::new(),
            el3state_ctx: EL3State::new(),
            el1_sysregs_ctx: EL1SysRegs::new(),
            fpregs_ctx: FPRegs::new(),
        }
    }
}

// The following function initializes the cpu_context 'ctx' for
// first use, and sets the initial entrypoint state as specified by the
// entry_point_info structure.
//
// The security state to initialize is determined by the SECURE attribute
// of the entry_point_info.
//
// The EE and ST attributes are used to configure the endianness and secure
// timer availability for the new execution context.
//
// To prepare the register state for entry call cm_prepare_el3_exit() and
// el3_exit(). For Secure-EL1 cm_prepare_el3_exit() is equivalent to
// cm_e1_sysreg_context_restore().
pub fn setup_context(ctx: &mut CPUContext, ep: EntryPointInfo) {}

// The following function initializes the cpu_context for the current CPU
// for first use, and sets the initial entrypoint state as specified by the
// entry_point_info structure.
pub fn init_context(idx: usize, ep: EntryPointInfo) {}
