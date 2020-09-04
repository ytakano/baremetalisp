use core::mem::size_of;
use core::ptr::{copy_nonoverlapping, read_volatile, write_volatile};

use crate::aarch64::cache;
use crate::driver::arm::gic;

// GICv2 specific Distributor interface register offsets and constants.
const GICD_ITARGETSR: usize = 0x800;
const GICD_SGIR: usize = 0xF00;
const GICD_CPENDSGIR: usize = 0xF10;
const GICD_SPENDSGIR: usize = 0xF20;
const GICD_PIDR2_GICV2: usize = 0xFE8;

// GICv2 specific CPU interface register offsets and constants.

// Physical CPU Interface registers
const GICC_CTLR: usize = 0x0;
const GICC_PMR: usize = 0x4;
const GICC_BPR: usize = 0x8;
const GICC_IAR: usize = 0xC;
const GICC_EOIR: usize = 0x10;
const GICC_RPR: usize = 0x14;
const GICC_HPPIR: usize = 0x18;
const GICC_AHPPIR: usize = 0x28;
const GICC_IIDR: usize = 0xFC;
const GICC_DIR: usize = 0x1000;
const GICC_PRIODROP: usize = GICC_EOIR;

// GICC_CTLR bit definitions
const EOI_MODE_NS: u32 = 1 << 10;
const EOI_MODE_S: u32 = 1 << 9;
const IRQ_BYP_DIS_GRP1: u32 = 1 << 8;
const FIQ_BYP_DIS_GRP1: u32 = 1 << 7;
const IRQ_BYP_DIS_GRP0: u32 = 1 << 6;
const FIQ_BYP_DIS_GRP0: u32 = 1 << 5;
const CBPR: u32 = 1 << 4;
const FIQ_EN_SHIFT: u32 = 3;
const FIQ_EN_BIT: u32 = 1 << FIQ_EN_SHIFT;
const ACK_CTL: u32 = 1 << 2;

const ITARGETSR_SHIFT: u32 = 2;
const GIC_TARGET_CPU_MASK: u32 = 0xff;

const CPENDSGIR_SHIFT: u32 = 2;
const SPENDSGIR_SHIFT: u32 = CPENDSGIR_SHIFT;

const SGIR_TGTLSTFLT_SHIFT: u32 = 24;
const SGIR_TGTLSTFLT_MASK: u32 = 0x3;
const SGIR_TGTLST_SHIFT: u32 = 16;
const SGIR_TGTLST_MASK: u32 = 0xff;
const SGIR_INTID_MASK: u32 = 0xf;

// Interrupt group definitions
const GICV2_INTR_GROUP0: u32 = 0;
const GICV2_INTR_GROUP1: u32 = 1;

// GICD_CTLR bit definitions
pub(crate) const CTLR_ENABLE_G1_SHIFT: u32 = 1;
pub(crate) const CTLR_ENABLE_G1_MASK: u32 = 0x1;
pub(crate) const CTLR_ENABLE_G1_BIT: u32 = 1 << CTLR_ENABLE_G1_SHIFT;

static mut DRIVER_DATA: GICv2DriverData = GICv2DriverData {
    gicd_base: 0,
    gicc_base: 0,
    target_masks: &NIL_U32,
    interrupt_props: &NIL_INT_PROP,
};

const NIL_U32: [u32; 0] = [];
const NIL_INT_PROP: [InterruptProp; 0] = [];

pub struct InterruptProp {
    pub intr_num: u32,
    pub intr_pri: u32,
    pub intr_grp: u32,
    pub intr_cfg: u32,
}

/// This structure describes some of the implementation defined attributes of
/// the GICv2 IP. It is used by the platform port to specify these attributes
/// in order to initialize the GICv2 driver. The attributes are described
/// below.
///
/// The 'gicd_base' field contains the base address of the Distributor interface
/// programmer's view.
///
/// The 'gicc_base' field contains the base address of the CPU Interface
/// programmer's view.
///
/// The 'target_masks' is a pointer to an array containing 'target_masks_num'
/// elements. The GIC driver will populate the array with per-PE target mask to
/// use to when targeting interrupts.
///
/// The 'interrupt_props' field is a pointer to an array that enumerates secure
/// interrupts and their properties. If this field is not NULL, both
/// 'g0_interrupt_array' and 'g1s_interrupt_array' fields are ignored.
pub struct GICv2DriverData {
    pub gicd_base: usize,
    pub gicc_base: usize,
    pub target_masks: &'static [u32],
    pub interrupt_props: &'static [InterruptProp],
}

impl GICv2DriverData {
    pub fn new_gicd_gicc(gicd_base: usize, gicc_base: usize) -> GICv2DriverData {
        GICv2DriverData {
            gicd_base: gicd_base,
            gicc_base: gicc_base,
            target_masks: &NIL_U32,
            interrupt_props: &NIL_INT_PROP,
        }
    }
}

// GIC Distributor interface accessors for reading entire registers

fn gicd_read_pidr2(base: usize) -> u32 {
    let ptr = (base + GICD_PIDR2_GICV2) as *const u32;
    unsafe { read_volatile(ptr) }
}

// GIC Distributor interface accessors for writing entire registers

fn gicd_set_itargetsr(base: usize, id: u32, target: u32) {
    let val = (target & GIC_TARGET_CPU_MASK) as u8;
    let ptr = (base + GICD_ITARGETSR + id as usize) as *mut u8;
    unsafe {
        write_volatile(ptr, val);
    }
}

/// Accessor to read the GIC Distributor ITARGETSR corresponding to the
/// interrupt `id`, 4 interrupt IDs at a time.
fn gicd_read_itargetsr(base: usize, id: u32) -> u32 {
    let n = id >> ITARGETSR_SHIFT;
    let ptr = (base + GICD_ITARGETSR + (n << 2) as usize) as *const u32;
    unsafe { read_volatile(ptr) }
}

/// Initialize the ARM GICv2 driver with the provided platform inputs
pub fn driver_init(driver_data: &GICv2DriverData) {
    // Ensure that this is a GICv2 system
    let pidr2 = gicd_read_pidr2(driver_data.gicd_base);
    let ver = (pidr2 >> gic::PIDR2_ARCH_REV_SHIFT) & gic::PIDR2_ARCH_REV_MASK;

    // GICv1 with security extension complies with trusted firmware
    // GICv2 driver as far as virtualization and few tricky power
    // features are not used. GICv2 features that are not supported
    // by GICv1 with Security Extensions are:
    // - virtual interrupt support.
    // - wake up events.
    // - writeable GIC state register (for power sequences)
    // - interrupt priority drop.
    // - interrupt signal bypass.
    if !(ver == gic::ARCH_REV_GICV1 || ver == gic::ARCH_REV_GICV2) {
        panic!("GIC version is incompatible with GICv2");
    }

    unsafe {
        copy_nonoverlapping(driver_data, &mut DRIVER_DATA, 1);
        cache::clean_invalidate(&mut DRIVER_DATA, size_of::<GICv2DriverData>());
    }
}

fn get_gicd_base() -> usize {
    unsafe { read_volatile(&DRIVER_DATA.gicd_base) }
}

fn get_gicc_base() -> usize {
    unsafe { read_volatile(&DRIVER_DATA.gicc_base) }
}

fn get_interrupt_props() -> &'static [InterruptProp] {
    unsafe { read_volatile(&DRIVER_DATA.interrupt_props) }
}

/// Global gic distributor init which will be done by the primary cpu after a
/// cold boot. It marks out the secure SPIs, PPIs & SGIs and enables them. It
/// then enables the secure GIC distributor interface.
pub fn distif_init() {
    // Disable the distributor before going further
    let base = get_gicd_base();
    let ctlr = gic::gicd_read_ctlr(base);
    gic::gicd_write_ctlr(base, ctlr & !(gic::CTLR_ENABLE_G0_BIT | CTLR_ENABLE_G1_BIT));

    // Set the default attribute of all SPIs
    spis_configure_defaults(base);

    secure_spis_configure_props(base, get_interrupt_props());

    // Re-enable the secure SPIs now that they have been configured
    gic::gicd_write_ctlr(base, ctlr | gic::CTLR_ENABLE_G0_BIT);
}

/// Helper function to configure the default attributes of SPIs.
fn spis_configure_defaults(gicd_base: usize) {
    let num_ints = gic::gicd_read_typer(gicd_base);
    let num_ints = num_ints & gic::TYPER_IT_LINES_NO_MASK;
    let num_ints = (num_ints + 1) << 5;

    // Treat all SPIs as G1NS by default. The number of interrupts is
    // calculated as 32 * (IT_LINES + 1). We do 32 at a time.
    for index in gic::MIN_SPI_ID..num_ints {
        gic::gicd_write_igroupr(gicd_base, index, !0);
    }

    // Setup the default SPI priorities doing four at a time
    for index in gic::MIN_SPI_ID..num_ints {
        gic::gicd_write_ipriorityr(gicd_base, index, gic::GICD_IPRIORITYR_DEF_VAL);
    }

    // Treat all SPIs as level triggered by default, 16 at a time
    for index in gic::MIN_SPI_ID..num_ints {
        gic::gicd_write_icfgr(gicd_base, index, 0);
    }
}

/// Get the current CPU bit mask from GICD_ITARGETSR0
fn gicd_get_cpuif_id(base: usize) -> u32 {
    let val = gicd_read_itargetsr(base, 0);
    val & GIC_TARGET_CPU_MASK
}

/// Helper function to configure properties of secure G0 SPIs.
fn secure_spis_configure_props(gicd_base: usize, interrupt_props: &'static [InterruptProp]) {
    for prop_desc in interrupt_props {
        if prop_desc.intr_num < gic::MIN_SPI_ID {
            continue;
        }

        // Configure this interrupt as a secure interrupt
        if prop_desc.intr_grp != GICV2_INTR_GROUP0 {
            panic!("invalid intr_grp");
        }
        gic::gicd_clr_igroupr(gicd_base, prop_desc.intr_num);

        // Set the priority of this interrupt
        gic::gicd_set_ipriorityr(gicd_base, prop_desc.intr_num, prop_desc.intr_cfg);

        // Target the secure interrupts to primary CPU
        gicd_set_itargetsr(gicd_base, prop_desc.intr_num, gicd_get_cpuif_id(gicd_base));

        // Set interrupt configuration
        gic::gicd_set_icfgr(gicd_base, prop_desc.intr_num, prop_desc.intr_cfg);

        // Enable this interrupt
        gic::gicd_set_isenabler(gicd_base, prop_desc.intr_num);
    }
}

/// Per cpu gic distributor setup which will be done by all cpus after a cold
/// boot/hotplug. This marks out the secure SPIs and PPIs & enables them.
pub fn pcpu_distif_init() {
    let base = get_gicd_base();
    if base == 0 {
        panic!("gicd_base is 0");
    }

    secure_ppi_sgi_setup_props(base, get_interrupt_props());

    // Enable G0 interrupts if not already
    let ctlr = gic::gicd_read_ctlr(base);
    if ctlr & gic::CTLR_ENABLE_G0_BIT == 0 {
        gic::gicd_write_ctlr(base, ctlr | gic::CTLR_ENABLE_G0_BIT);
    }
}

/// Helper function to configure properties of secure G0 SGIs and PPIs.
fn secure_ppi_sgi_setup_props(gicd_base: usize, interrupt_props: &[InterruptProp]) {
    // Disable all SGIs (imp. def.)/PPIs before configuring them. This is a
    // more scalable approach as it avoids clearing the enable bits in the
    // GICD_CTLR.
    gic::gicd_write_icenabler(gicd_base, 0, !0);

    // Setup the default PPI/SGI priorities doing four at a time
    for i in (0..gic::MIN_SPI_ID).step_by(4) {
        gic::gicd_write_ipriorityr(gicd_base, i, gic::GICD_IPRIORITYR_DEF_VAL);
    }

    let mut sec_ppi_sgi_mask = 0;
    for prop_desc in interrupt_props {
        if prop_desc.intr_num >= gic::MIN_SPI_ID {
            continue;
        }

        // Configure this interrupt as a secure interrupt
        if prop_desc.intr_grp != GICV2_INTR_GROUP0 {
            panic!("not secure interrupt")
        }

        // Set interrupt configuration for PPIs. Configuration for SGIs
        // are ignored.
        if prop_desc.intr_num >= gic::MIN_PPI_ID && prop_desc.intr_num < gic::MIN_SPI_ID {
            gic::gicd_set_icfgr(gicd_base, prop_desc.intr_num, prop_desc.intr_cfg);
        }

        // We have an SGI or a PPI. They are Group0 at reset
        sec_ppi_sgi_mask |= 1 << prop_desc.intr_num;

        // Set the priority of this interrupt
        gic::gicd_set_ipriorityr(gicd_base, prop_desc.intr_num, prop_desc.intr_pri);
    }

    // Invert the bitmask to create a mask for non-secure PPIs and SGIs.
    // Program the GICD_IGROUPR0 with this bit mask.
    gic::gicd_write_igroupr(gicd_base, 0, !sec_ppi_sgi_mask);

    // Enable the Group 0 SGIs and PPIs
    gic::gicd_write_isenabler(gicd_base, 0, sec_ppi_sgi_mask);
}

/// Enable secure interrupts and use FIQs to route them. Disable legacy bypass
/// and set the priority mask register to allow all interrupts to trickle in.
pub fn cpuif_enable() {
    // Enable the Group 0 interrupts, FIQEn and disable Group 0/1
    // bypass.
    let val = gic::CTLR_ENABLE_G0_BIT
        | FIQ_EN_BIT
        | FIQ_BYP_DIS_GRP0
        | IRQ_BYP_DIS_GRP0
        | FIQ_BYP_DIS_GRP1
        | IRQ_BYP_DIS_GRP1;

    // Program the idle priority in the PMR
    let base = get_gicc_base();
    gicc_write_pmr(base, gic::GIC_PRI_MASK);
    gicc_write_ctlr(base, val);
}

fn gicc_write_pmr(base: usize, val: u32) {
    let ptr = (base + GICC_PMR) as *mut u32;
    unsafe { write_volatile(ptr, val) };
}

fn gicc_write_ctlr(base: usize, val: u32) {
    let ptr = (base + GICC_CTLR) as *mut u32;
    unsafe { write_volatile(ptr, val) };
}

fn gicc_read_ctlr(base: usize) -> u32 {
    let ptr = (base + GICC_CTLR) as *mut u32;
    unsafe { read_volatile(ptr) }
}

/// Place the cpu interface in a state where it can never make a cpu exit wfi as
/// as result of an asserted interrupt. This is critical for powering down a cpu
pub fn cpuif_disable() {
    // Disable secure, non-secure interrupts and disable their bypass
    let base = get_gicc_base();
    let val = gicc_read_ctlr(base) & !(gic::CTLR_ENABLE_G0_BIT | CTLR_ENABLE_G1_BIT)
        | FIQ_BYP_DIS_GRP1
        | FIQ_BYP_DIS_GRP0
        | IRQ_BYP_DIS_GRP0
        | IRQ_BYP_DIS_GRP1;
    gicc_write_ctlr(base, val);
}
