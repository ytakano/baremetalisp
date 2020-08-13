// SCR definitions
pub const SCR_RES1_BITS: u64 = (1 << 4) | (1 << 5);
pub const SCR_TWEDEL_SHIFT: u64 = 30;
pub const SCR_TWEDEL_MASK: u64 = 0xf;
pub const SCR_TWEDEN_BIT: u64 = 1 << 29;
pub const SCR_ECVEN_BIT: u64 = 1 << 28;
pub const SCR_FGTEN_BIT: u64 = 1 << 27;
pub const SCR_ATA_BIT: u64 = 1 << 26;
pub const SCR_FIEN_BIT: u64 = 1 << 21;
pub const SCR_EEL2_BIT: u64 = 1 << 18;
pub const SCR_API_BIT: u64 = 1 << 17;
pub const SCR_APK_BIT: u64 = 1 << 16;
pub const SCR_TERR_BIT: u64 = 1 << 15;
pub const SCR_TWE_BIT: u64 = 1 << 13;
pub const SCR_TWI_BIT: u64 = 1 << 12;
pub const SCR_ST_BIT: u64 = 1 << 11;
pub const SCR_RW_BIT: u64 = 1 << 10;
pub const SCR_SIF_BIT: u64 = 1 << 9;
pub const SCR_HCE_BIT: u64 = 1 << 8;
pub const SCR_SMD_BIT: u64 = 1 << 7;
pub const SCR_EA_BIT: u64 = 1 << 3;
pub const SCR_FIQ_BIT: u64 = 1 << 2;
pub const SCR_IRQ_BIT: u64 = 1 << 1;
pub const SCR_NS_BIT: u64 = 1 << 0;
pub const SCR_VALID_BIT_MASK: u64 = 0x2f8f;
pub const SCR_RESET_VAL: u64 = SCR_RES1_BITS;

// SCTLR definitions
pub const SCTLR_EL2_RES1: u64 = (1 << 29)
    | (1 << 28)
    | (1 << 23)
    | (1 << 22)
    | (1 << 18)
    | (1 << 16)
    | (1 << 11)
    | (1 << 5)
    | (1 << 4);
pub const SCTLR_EL1_RES1: u64 =
    (1 << 29) | (1 << 28) | (1 << 23) | (1 << 22) | (1 << 20) | (1 << 11);
pub const SCTLR_AARCH32_EL1_RES1: u64 = (1 << 23) | (1 << 22) | (1 << 11) | (1 << 4) | (1 << 3);
pub const SCTLR_EL3_RES1: u64 = (1 << 29)
    | (1 << 28)
    | (1 << 23)
    | (1 << 22)
    | (1 << 18)
    | (1 << 16)
    | (1 << 11)
    | (1 << 5)
    | (1 << 4);
pub const SCTLR_M_BIT: u64 = 1 << 0;
pub const SCTLR_A_BIT: u64 = 1 << 1;
pub const SCTLR_C_BIT: u64 = 1 << 2;
pub const SCTLR_SA_BIT: u64 = 1 << 3;
pub const SCTLR_SA0_BIT: u64 = 1 << 4;
pub const SCTLR_CP15BEN_BIT: u64 = 1 << 5;
pub const SCTLR_ITD_BIT: u64 = 1 << 7;
pub const SCTLR_SED_BIT: u64 = 1 << 8;
pub const SCTLR_UMA_BIT: u64 = 1 << 9;
pub const SCTLR_I_BIT: u64 = 1 << 12;
pub const SCTLR_ENDB_BIT: u64 = 1 << 13;
pub const SCTLR_DZE_BIT: u64 = 1 << 14;
pub const SCTLR_UCT_BIT: u64 = 1 << 15;
pub const SCTLR_NTWI_BIT: u64 = 1 << 16;
pub const SCTLR_NTWE_BIT: u64 = 1 << 18;
pub const SCTLR_WXN_BIT: u64 = 1 << 19;
pub const SCTLR_UWXN_BIT: u64 = 1 << 20;
pub const SCTLR_IESB_BIT: u64 = 1 << 21;
pub const SCTLR_E0E_BIT: u64 = 1 << 24;
pub const SCTLR_EE_BIT: u64 = 1 << 25;
pub const SCTLR_UCI_BIT: u64 = 1 << 26;
pub const SCTLR_ENDA_BIT: u64 = 1 << 27;
pub const SCTLR_ENIB_BIT: u64 = 1 << 30;
pub const SCTLR_ENIA_BIT: u64 = 1 << 31;
pub const SCTLR_BT0_BIT: u64 = 1 << 35;
pub const SCTLR_BT1_BIT: u64 = 1 << 36;
pub const SCTLR_BT_BIT: u64 = 1 << 36;
pub const SCTLR_DSSBS_BIT: u64 = 1 << 44;
pub const SCTLR_RESET_VAL: u64 = SCTLR_EL3_RES1;

pub const MODE_SP_SHIFT: u64 = 0x0;
pub const MODE_SP_MASK: u64 = 0x1;
pub const MODE_SP_EL0: u64 = 0x0;
pub const MODE_SP_ELX: u64 = 0x1;

pub const MODE_RW_SHIFT: u64 = 0x4;
pub const MODE_RW_MASK: u64 = 0x1;
pub const MODE_RW_64: u64 = 0x0;
pub const MODE_RW_32: u64 = 0x1;

pub const MODE_EL_SHIFT: u64 = 0x2;
pub const MODE_EL_MASK: u64 = 0x3;
pub const MODE_EL_WIDTH: u64 = 0x2;
pub const MODE_EL3: u64 = 0x3;
pub const MODE_EL2: u64 = 0x2;
pub const MODE_EL1: u64 = 0x1;
pub const MODE_EL0: u64 = 0x0;

pub const MODE32_SHIFT: u64 = 0;
pub const MODE32_MASK: u64 = 0xf;
pub const MODE32_USR: u64 = 0x0;
pub const MODE32_FIQ: u64 = 0x1;
pub const MODE32_IRQ: u64 = 0x2;
pub const MODE32_SVC: u64 = 0x3;
pub const MODE32_MON: u64 = 0x6;
pub const MODE32_ABT: u64 = 0x7;
pub const MODE32_HYP: u64 = 0xa;
pub const MODE32_UND: u64 = 0xb;
pub const MODE32_SYS: u64 = 0xf;

// CPSR/SPSR definitions
pub const DAIF_FIQ_BIT: u64 = 1 << 0;
pub const DAIF_IRQ_BIT: u64 = 1 << 1;
pub const DAIF_ABT_BIT: u64 = 1 << 2;
pub const DAIF_DBG_BIT: u64 = 1 << 3;
pub const SPSR_DAIF_SHIFT: u64 = 6;
pub const SPSR_DAIF_MASK: u64 = 0xf;

pub const SPSR_AIF_SHIFT: u64 = 6;
pub const SPSR_AIF_MASK: u64 = 0x7;

pub const SPSR_E_SHIFT: u64 = 9;
pub const SPSR_E_MASK: u64 = 0x1;
pub const SPSR_E_LITTLE: u64 = 0x0;
pub const SPSR_E_BIG: u64 = 0x1;

pub const SPSR_T_SHIFT: u64 = 5;
pub const SPSR_T_MASK: u64 = 0x1;
pub const SPSR_T_ARM: u64 = 0x0;
pub const SPSR_T_THUMB: u64 = 0x1;

pub const SPSR_M_SHIFT: u64 = 4;
pub const SPSR_M_MASK: u64 = 0x1;
pub const SPSR_M_AARCH64: u64 = 0x0;
pub const SPSR_M_AARCH32: u64 = 0x1;

pub const SPSR_EL_SHIFT: u64 = 2;
pub const SPSR_EL_WIDTH: u64 = 2;

pub const SPSR_SSBS_BIT_AARCH64: u64 = 1 << 12;
pub const SPSR_SSBS_BIT_AARCH32: u64 = 1 << 23;

pub const DISABLE_ALL_EXCEPTIONS: u64 = DAIF_FIQ_BIT | DAIF_IRQ_BIT | DAIF_ABT_BIT | DAIF_DBG_BIT;

pub const DISABLE_INTERRUPTS: u64 = DAIF_FIQ_BIT | DAIF_IRQ_BIT;

// ID_AA64PFR0_EL1 definitions
pub const ID_AA64PFR0_EL0_SHIFT: u64 = 0;
pub const ID_AA64PFR0_EL1_SHIFT: u64 = 4;
pub const ID_AA64PFR0_EL2_SHIFT: u64 = 8;
pub const ID_AA64PFR0_EL3_SHIFT: u64 = 12;
pub const ID_AA64PFR0_AMU_SHIFT: u64 = 44;
pub const ID_AA64PFR0_AMU_MASK: u64 = 0xf;
pub const ID_AA64PFR0_ELX_MASK: u64 = 0xf;
pub const ID_AA64PFR0_GIC_SHIFT: u64 = 24;
pub const ID_AA64PFR0_GIC_WIDTH: u64 = 4;
pub const ID_AA64PFR0_GIC_MASK: u64 = 0xf;
pub const ID_AA64PFR0_SVE_SHIFT: u64 = 32;
pub const ID_AA64PFR0_SVE_MASK: u64 = 0xf;
pub const ID_AA64PFR0_SEL2_SHIFT: u64 = 36;
pub const ID_AA64PFR0_SEL2_MASK: u64 = 0xf;
pub const ID_AA64PFR0_MPAM_SHIFT: u64 = 40;
pub const ID_AA64PFR0_MPAM_MASK: u64 = 0xf;
pub const ID_AA64PFR0_DIT_SHIFT: u64 = 48;
pub const ID_AA64PFR0_DIT_MASK: u64 = 0xf;
pub const ID_AA64PFR0_DIT_LENGTH: u64 = 4;
pub const ID_AA64PFR0_DIT_SUPPORTED: u64 = 1;
pub const ID_AA64PFR0_CSV2_SHIFT: u64 = 56;
pub const ID_AA64PFR0_CSV2_MASK: u64 = 0xf;
pub const ID_AA64PFR0_CSV2_LENGTH: u64 = 4;

// ID_AA64PFR1_EL1 definitions
pub const ID_AA64PFR1_EL1_SSBS_SHIFT: u64 = 4;
pub const ID_AA64PFR1_EL1_SSBS_MASK: u64 = 0xf;

pub const SSBS_UNAVAILABLE: u64 = 0; // No architectural SSBS support

pub const ID_AA64PFR1_EL1_BT_SHIFT: u64 = 0;
pub const ID_AA64PFR1_EL1_BT_MASK: u64 = 0xf;

pub const BTI_IMPLEMENTED: u64 = 1; // The BTI mechanism is implemented

pub const ID_AA64PFR1_EL1_MTE_SHIFT: u64 = 8;
pub const ID_AA64PFR1_EL1_MTE_MASK: u64 = 0xf;

pub const MTE_UNIMPLEMENTED: u64 = 0;
pub const MTE_IMPLEMENTED_EL0: u64 = 1; // MTE is only implemented at EL0
pub const MTE_IMPLEMENTED_ELX: u64 = 2; // MTE is implemented at all ELs

pub const ID_AA64PFR1_MPAM_FRAC_SHIFT: u64 = 16;
pub const ID_AA64PFR1_MPAM_FRAC_MASK: u64 = 0xf;

// ID_AA64MMFR0_EL1 definitions
pub const ID_AA64MMFR0_EL1_PARANGE_SHIFT: u64 = 0;
pub const ID_AA64MMFR0_EL1_PARANGE_MASK: u64 = 0xf;

pub const PARANGE_0000: u64 = 32;
pub const PARANGE_0001: u64 = 36;
pub const PARANGE_0010: u64 = 40;
pub const PARANGE_0011: u64 = 42;
pub const PARANGE_0100: u64 = 44;
pub const PARANGE_0101: u64 = 48;
pub const PARANGE_0110: u64 = 52;

pub const ID_AA64MMFR0_EL1_ECV_SHIFT: u64 = 60;
pub const ID_AA64MMFR0_EL1_ECV_MASK: u64 = 0xf;
pub const ID_AA64MMFR0_EL1_ECV_NOT_SUPPORTED: u64 = 0x0;
pub const ID_AA64MMFR0_EL1_ECV_SUPPORTED: u64 = 0x1;
pub const ID_AA64MMFR0_EL1_ECV_SELF_SYNCH: u64 = 0x2;

pub const ID_AA64MMFR0_EL1_FGT_SHIFT: u64 = 56;
pub const ID_AA64MMFR0_EL1_FGT_MASK: u64 = 0xf;
pub const ID_AA64MMFR0_EL1_FGT_SUPPORTED: u64 = 0x1;
pub const ID_AA64MMFR0_EL1_FGT_NOT_SUPPORTED: u64 = 0x0;

pub const ID_AA64MMFR0_EL1_TGRAN4_SHIFT: u64 = 28;
pub const ID_AA64MMFR0_EL1_TGRAN4_MASK: u64 = 0xf;
pub const ID_AA64MMFR0_EL1_TGRAN4_SUPPORTED: u64 = 0x0;
pub const ID_AA64MMFR0_EL1_TGRAN4_NOT_SUPPORTED: u64 = 0xf;

pub const ID_AA64MMFR0_EL1_TGRAN64_SHIFT: u64 = 24;
pub const ID_AA64MMFR0_EL1_TGRAN64_MASK: u64 = 0xf;
pub const ID_AA64MMFR0_EL1_TGRAN64_SUPPORTED: u64 = 0x0;
pub const ID_AA64MMFR0_EL1_TGRAN64_NOT_SUPPORTED: u64 = 0xf;

pub const ID_AA64MMFR0_EL1_TGRAN16_SHIFT: u64 = 20;
pub const ID_AA64MMFR0_EL1_TGRAN16_MASK: u64 = 0xf;
pub const ID_AA64MMFR0_EL1_TGRAN16_SUPPORTED: u64 = 0x1;
pub const ID_AA64MMFR0_EL1_TGRAN16_NOT_SUPPORTED: u64 = 0x0;

// ID_AA64MMFR1_EL1 definitions
pub const ID_AA64MMFR1_EL1_TWED_SHIFT: u64 = 32;
pub const ID_AA64MMFR1_EL1_TWED_MASK: u64 = 0xf;
pub const ID_AA64MMFR1_EL1_TWED_SUPPORTED: u64 = 0x1;
pub const ID_AA64MMFR1_EL1_TWED_NOT_SUPPORTED: u64 = 0x0;

pub fn spsr64(el: u64, sp: u64, daif: u64) -> u64 {
    ((MODE_RW_64 << MODE_RW_SHIFT)
        | (((el) & MODE_EL_MASK) << MODE_EL_SHIFT)
        | (((sp) & MODE_SP_MASK) << MODE_SP_SHIFT)
        | (((daif) & SPSR_DAIF_MASK) << SPSR_DAIF_SHIFT))
        & (!(SPSR_SSBS_BIT_AARCH64))
}

pub fn spsr32(mode: u64, isa: u64, endian: u64, aif: u64) -> u64 {
    ((MODE_RW_32 << MODE_RW_SHIFT)
        | (((mode) & MODE32_MASK) << MODE32_SHIFT)
        | (((isa) & SPSR_T_MASK) << SPSR_T_SHIFT)
        | (((endian) & SPSR_E_MASK) << SPSR_E_SHIFT)
        | (((aif) & SPSR_AIF_MASK) << SPSR_AIF_SHIFT))
        & (!(SPSR_SSBS_BIT_AARCH32))
}

pub fn get_affinity_lv0() -> u64 {
    let mpidr: u64;
    unsafe {
        asm!("mrs {}, mpidr_el1", lateout(reg) mpidr);
    }

    mpidr & 0xFF
}

pub fn get_affinity_lv1() -> u64 {
    let mpidr: u64;
    unsafe {
        asm!("mrs {}, mpidr_el1", lateout(reg) mpidr);
    }

    (mpidr >> 8) & 0xFF
}

pub fn get_current_el() -> u32 {
    let el: u64;
    unsafe {
        asm!("mrs {}, CurrentEL", lateout(reg) el);
    }
    ((el >> 2) & 0x3) as u32
}

pub fn get_scr_el3() -> u64 {
    let scr_el3;
    unsafe {
        asm!("mrs {}, scr_el3", lateout(reg) scr_el3);
    }
    scr_el3
}

pub fn get_sctlr_el1() -> u64 {
    let sctlr_el1;
    unsafe {
        asm!("mrs {}, sctlr_el1", lateout(reg) sctlr_el1);
    }
    sctlr_el1
}

pub fn get_sctlr_el2() -> u64 {
    let sctlr_el2;
    unsafe {
        asm!("mrs {}, sctlr_el2", lateout(reg) sctlr_el2);
    }
    sctlr_el2
}

pub fn get_actlr_el1() -> u64 {
    let actlr_el1;
    unsafe {
        asm!("mrs {}, actlr_el1", lateout(reg) actlr_el1);
    }
    actlr_el1
}

pub fn get_id_aa64pfr1_el1() -> u64 {
    let id_aa64pfr1_el1;
    unsafe {
        asm!("mrs {}, id_aa64pfr1_el1", lateout(reg) id_aa64pfr1_el1);
    }
    id_aa64pfr1_el1
}

pub fn get_id_aa64mmfr0_el1() -> u64 {
    let id_aa64mmfr0_el1;
    unsafe {
        asm!("mrs {}, id_aa64mmfr0_el1", lateout(reg) id_aa64mmfr0_el1);
    }
    id_aa64mmfr0_el1
}

pub fn get_id_aa64mmfr1_el1() -> u64 {
    let id_aa64mmfr1_el1;
    unsafe {
        asm!("mrs {}, id_aa64mmfr1_el1", lateout(reg) id_aa64mmfr1_el1);
    }
    id_aa64mmfr1_el1
}

pub fn get_armv8_5_mte_support() -> u64 {
    (get_id_aa64pfr1_el1() >> ID_AA64PFR1_EL1_MTE_SHIFT) & ID_AA64PFR1_EL1_MTE_MASK
}

pub fn is_armv8_6_twed_present() -> bool {
    ((get_id_aa64mmfr1_el1() >> ID_AA64MMFR1_EL1_TWED_SHIFT) & ID_AA64MMFR1_EL1_TWED_MASK)
        == ID_AA64MMFR1_EL1_TWED_SUPPORTED
}

pub fn is_armv8_6_fgt_present() -> bool {
    ((get_id_aa64mmfr0_el1() >> ID_AA64MMFR0_EL1_FGT_SHIFT) & ID_AA64MMFR0_EL1_FGT_MASK)
        == ID_AA64MMFR0_EL1_FGT_SUPPORTED
}

pub fn get_armv8_6_ecv_support() -> u64 {
    (get_id_aa64mmfr0_el1() >> ID_AA64MMFR0_EL1_ECV_SHIFT) & ID_AA64MMFR0_EL1_ECV_MASK
}

/// sev
pub fn send_event() {
    unsafe {
        asm!("sev");
    }
}

/// sevl
pub fn send_event_local() {
    unsafe {
        asm!("sevl");
    }
}

/// wfe
pub fn wait_event() {
    unsafe {
        asm!("wfe");
    }
}

/// dmb st
pub fn dmb_st() {
    unsafe {
        asm!("dmb st");
    }
}

/// dmb ld
pub fn dmb_ld() {
    unsafe {
        asm!("dmb ld");
    }
}

/// dmb sy
pub fn dmb_sy() {
    unsafe {
        asm!("dmb sy");
    }
}

pub fn start_non_primary() {
    if cfg!(feature = "raspi3") {
        unsafe {
            asm!(
                "mov {0}, #0xe0
                 ldr {1}, =_start
                 str {1}, [{0}]
                 str {1}, [{0},  8] // core #2
                 str {1}, [{0}, 16] // core #3",
            lateout(reg) _,
            lateout(reg) _
            );
        }
    }
}
