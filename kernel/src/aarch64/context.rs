use super::cpu;
use crate::driver::topology::CORE_COUNT;
use crate::psci::ep_info::EntryPointInfo;

use core::ptr::{copy_nonoverlapping, write_volatile};

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
pub fn setup_context(ctx: &mut CPUContext, ep: EntryPointInfo) {
    let is_secure = ep.is_secure();

    // Clear any residual register values from the context
    unsafe {
        copy_nonoverlapping(&CPUContext::new(), ctx, 1);
    }

    // SCR_EL3 was initialised during reset sequence in macro
    // el3_arch_init_common. This code modifies the SCR_EL3 fields that
    // affect the next EL.
    //
    // The following fields are initially set to zero and then updated to
    // the required value depending on the state of the SPSR_EL3 and the
    // Security state and entrypoint attributes of the next EL.
    let mut scr_el3 = cpu::get_scr_el3()
        & !(cpu::SCR_NS_BIT
            | cpu::SCR_RW_BIT
            | cpu::SCR_FIQ_BIT
            | cpu::SCR_IRQ_BIT
            | cpu::SCR_ST_BIT
            | cpu::SCR_HCE_BIT);

    // SCR_NS: Set the security state of the next EL.
    if !is_secure {
        scr_el3 |= cpu::SCR_NS_BIT;
    }

    // SCR_EL3.RW: Set the execution state, AArch32 or AArch64, for next
    //  Exception level as specified by SPSR.
    if ep.is_mode_rw64() {
        scr_el3 |= cpu::SCR_RW_BIT;
    }

    // SCR_EL3.ST: Traps Secure EL1 accesses to the Counter-timer Physical
    //  Secure timer registers to EL3, from AArch64 state only, if specified
    //  by the entrypoint attributes.
    if ep.is_st_enable() {
        scr_el3 |= cpu::SCR_ST_BIT;
    }

    // skip some configurations
    // see https://github.com/ARM-software/arm-trusted-firmware/blob/8f09da46e263cdb97f01edce449aa5b769cca2f5/lib/el3_runtime/aarch64/context_mgmt.c#L111-L178

    let mte = cpu::get_armv8_5_mte_support();
    if mte == cpu::MTE_IMPLEMENTED_EL0 {
        // Can enable MTE across both worlds as no MTE registers are
        // used
        scr_el3 |= cpu::SCR_ATA_BIT;
    } else if mte == cpu::MTE_IMPLEMENTED_ELX && !is_secure {
        // Can only enable MTE in Non-Secure world without register
        // saving
        scr_el3 |= cpu::SCR_ATA_BIT;
    }

    // SCR_EL3.HCE: Enable HVC instructions if next execution state is
    // AArch64 and next EL is EL2, or if next execution state is AArch32 and
    // next mode is Hyp.
    // SCR_EL3.FGTEn: Enable Fine Grained Virtualization Traps under the
    // same conditions as HVC instructions and when the processor supports
    // ARMv8.6-FGT.
    // SCR_EL3.ECVEn: Enable Enhanced Counter Virtualization (ECV)
    // CNTPOFF_EL2 register under the same conditions as HVC instructions
    // and when the processor supports ECV.
    if (ep.is_mode_rw64() && ep.get_el() == cpu::MODE_EL2)
        || (!ep.is_mode_rw64() && ep.get_m32() == cpu::MODE32_HYP)
    {
        if cpu::is_armv8_6_fgt_present() {
            scr_el3 |= cpu::SCR_FGTEN_BIT;
        }

        if cpu::get_armv8_6_ecv_support() == cpu::ID_AA64MMFR0_EL1_ECV_SELF_SYNCH {
            scr_el3 |= cpu::SCR_ECVEN_BIT;
        }

        scr_el3 |= cpu::SCR_HCE_BIT;
    }

    // Enable S-EL2 if the next EL is EL2 and security state is secure
    if is_secure && ep.get_el() == cpu::MODE_EL2 {
        if !ep.is_mode_rw64() {
            panic!("S-EL2 can not be used in AArch32.");
        }
        scr_el3 |= cpu::SCR_EEL2_BIT;
    }

    // Initialise SCTLR_EL1 to the reset value corresponding to the target
    // execution state setting all fields rather than relying of the hw.
    // Some fields have architecturally UNKNOWN reset values and these are
    // set to zero.
    //
    // SCTLR.EE: Endianness is taken from the entrypoint attributes.
    //
    // SCTLR.M, SCTLR.C and SCTLR.I: These fields must be zero (as
    //  required by PSCI specification)
    let mut sctlr_elx = if ep.is_big_endian() {
        cpu::SCTLR_EE_BIT
    } else {
        0
    };

    if ep.is_mode_rw64() {
        sctlr_elx |= cpu::SCTLR_EL1_RES1;
    } else {
        // If the target execution state is AArch32 then the following
        // fields need to be set.
        //
        // SCTRL_EL1.nTWE: Set to one so that EL0 execution of WFE
        //  instructions are not trapped to EL1.
        //
        // SCTLR_EL1.nTWI: Set to one so that EL0 execution of WFI
        //  instructions are not trapped to EL1.
        //
        // SCTLR_EL1.CP15BEN: Set to one to enable EL0 execution of the
        //  CP15DMB, CP15DSB, and CP15ISB instructions.
        sctlr_elx |= cpu::SCTLR_AARCH32_EL1_RES1
            | cpu::SCTLR_CP15BEN_BIT
            | cpu::SCTLR_NTWI_BIT
            | cpu::SCTLR_NTWE_BIT;
    }

    // If workaround of errata 764081 for Cortex-A75 is used then set
    // SCTLR_EL1.IESB to enable Implicit Error Synchronization Barrier.
    sctlr_elx = errata_a75_764081(sctlr_elx);

    // Enable WFE trap delay in SCR_EL3 if supported and configured
    // (for Armv8.6)
    // see https://github.com/ARM-software/arm-trusted-firmware/blob/8f09da46e263cdb97f01edce449aa5b769cca2f5/lib/el3_runtime/aarch64/context_mgmt.c#L256-L272
    // if cpu::is_armv8_6_twed_present() {}

    // Store the initialised SCTLR_EL1 value in the cpu_context - SCTLR_EL2
    // and other EL2 registers are set up by cm_prepare_ns_entry() as they
    // are not part of the stored cpu_context.
    unsafe {
        write_volatile(&mut ctx.el1_sysregs_ctx.sctlr_el1, sctlr_elx);
    }

    // Base the context ACTLR_EL1 on the current value, as it is
    // implementation defined. The context restore process will write
    // the value from the context to the actual register and can cause
    // problems for processor cores that don't expect certain bits to
    // be zero.
    let actlr_el1 = cpu::get_actlr_el1();
    unsafe {
        write_volatile(&mut ctx.el1_sysregs_ctx.actlr_el1, actlr_el1);
    }

    // Populate EL3 state so that we've the right context
    // before doing ERET
    unsafe {
        write_volatile(&mut ctx.el3state_ctx.scr_el3, scr_el3);
        write_volatile(&mut ctx.el3state_ctx.elr_el3, ep.pc as u64);
        write_volatile(&mut ctx.el3state_ctx.spsr_el3, ep.spsr);
    }

    // Store the X0-X7 value from the entrypoint into the context
    // Use memcpy as we are in control of the layout of the structures
    unsafe {
        write_volatile(&mut ctx.gpregx_ctx.x0, ep.args.arg0);
        write_volatile(&mut ctx.gpregx_ctx.x1, ep.args.arg1);
        write_volatile(&mut ctx.gpregx_ctx.x2, ep.args.arg2);
        write_volatile(&mut ctx.gpregx_ctx.x3, ep.args.arg3);
        write_volatile(&mut ctx.gpregx_ctx.x4, ep.args.arg4);
        write_volatile(&mut ctx.gpregx_ctx.x5, ep.args.arg5);
        write_volatile(&mut ctx.gpregx_ctx.x6, ep.args.arg6);
        write_volatile(&mut ctx.gpregx_ctx.x7, ep.args.arg7);
    }
}

// The following function initializes the cpu_context for the current CPU
// for first use, and sets the initial entrypoint state as specified by the
// entry_point_info structure.
pub fn init_context(idx: usize, ep: EntryPointInfo) {
    let ctx = if ep.is_secure() {
        unsafe { &mut CPU_CONTEXT_SECURE[idx] }
    } else {
        unsafe { &mut CPU_CONTEXT_NON_SECURE[idx] }
    };
    setup_context(ctx, ep);
}

#[cfg(feature = "ERRATA_A75_764081")]
fn errata_a75_764081(sctlr: u64) -> u64 {
    sctlr | cpu::SCTLR_IESB_BIT
}

#[cfg(not(feature = "ERRATA_A75_764081"))]
fn errata_a75_764081(sctlr: u64) -> u64 {
    sctlr
}
