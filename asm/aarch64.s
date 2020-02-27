.section .init, "x"
.global _start

_start:
    // disable all interrupt (daif at bits 9..6)
    msr     DAIFSet, #0x0f

    // read cpu id, stop slave cores
    mrs     x1, mpidr_el1
    and     x1, x1, #3
    cbz     x1, .L2

    // if cpu id > 0 then stop
.L1:
    wfe
    b       .L1

    // if cpu id == 0
.L2:
    // set stack before _start
    ldr     x1, =__stack_start
    mov     sp, x1

    // clear bss
    ldr     x1, =__bss_start
    ldr     w2, =__bss_size
.L3:
    cbz     w2, .L4
    str     xzr, [x1], #8
    sub     w2, w2, #1
    cbnz    w2, .L3

    // set exception vector
    ldr     x1, =exception_vector_el2
    msr     vbar_el1, x1

    mrs     x1, scr_el3
    and     x1, x1, #~(1 << 3) // EA
    and     x1, x1, #~(1 << 1) // IRQ
    and     x1, x1, #~(1 << 2) // FIQ
    MSR     scr_el3, x0

    // enable all interrupt
    mrs     x1, hcr_el2
    orr     x1, x1, #(1 << 5) // AMO
    orr     x1, x1, #(1 << 4) // IMO
    orr     x1, x1, #(1 << 3) // FMO
    msr     hcr_el2, x1
    msr     DAIFClr, #0xF

.L4:
    bl      entry
    b       _start

.macro CALL_WITH_CONTEXT handler
    // Make room on the stack for the exception context.
    sub     sp,  sp,  #16 * 17

    // Store all general purpose registers on the stack.
    stp     x0,  x1,  [sp, #16 * 0]
    stp     x2,  x3,  [sp, #16 * 1]
    stp     x4,  x5,  [sp, #16 * 2]
    stp     x6,  x7,  [sp, #16 * 3]
    stp     x8,  x9,  [sp, #16 * 4]
    stp     x10, x11, [sp, #16 * 5]
    stp     x12, x13, [sp, #16 * 6]
    stp     x14, x15, [sp, #16 * 7]
    stp     x16, x17, [sp, #16 * 8]
    stp     x18, x19, [sp, #16 * 9]
    stp     x20, x21, [sp, #16 * 10]
    stp     x22, x23, [sp, #16 * 11]
    stp     x24, x25, [sp, #16 * 12]
    stp     x26, x27, [sp, #16 * 13]
    stp     x28, x29, [sp, #16 * 14]

    // Add the exception link register (ELR_EL2) and the saved program status (SPSR_EL2).
    mrs     x1,  ELR_EL2
    mrs     x2,  SPSR_EL2

    stp     lr,  x1,  [sp, #16 * 15]
    str     w2,       [sp, #16 * 16]

    // x0 is the first argument for the function called through `\handler`.
    mov     x0,  sp

    // Call `\handler`.
    bl      \handler

    // After returning from exception handling code, replay the saved context and return via `eret`.
    b       exception_restore_context
.endm

//--------------------------------------------------------------------------------------------------
// Helper functions
//--------------------------------------------------------------------------------------------------
exception_restore_context:
    ldr     w19,      [sp, #16 * 16]
    ldp     lr,  x20, [sp, #16 * 15]

    msr     SPSR_EL2, x19
    msr     ELR_EL2,  x20

    ldp     x0,  x1,  [sp, #16 * 0]
    ldp     x2,  x3,  [sp, #16 * 1]
    ldp     x4,  x5,  [sp, #16 * 2]
    ldp     x6,  x7,  [sp, #16 * 3]
    ldp     x8,  x9,  [sp, #16 * 4]
    ldp     x10, x11, [sp, #16 * 5]
    ldp     x12, x13, [sp, #16 * 6]
    ldp     x14, x15, [sp, #16 * 7]
    ldp     x16, x17, [sp, #16 * 8]
    ldp     x18, x19, [sp, #16 * 9]
    ldp     x20, x21, [sp, #16 * 10]
    ldp     x22, x23, [sp, #16 * 11]
    ldp     x24, x25, [sp, #16 * 12]
    ldp     x26, x27, [sp, #16 * 13]
    ldp     x28, x29, [sp, #16 * 14]

    add     sp,  sp,  #16 * 17

    eret

    .balign 0x800
exception_vector_el2:
    // from the current EL using the current SP0
    CALL_WITH_CONTEXT curr_el_sp0_sync_el2
    .balign 0x80
    CALL_WITH_CONTEXT curr_el_sp0_irq_el2
    .balign 0x80
    CALL_WITH_CONTEXT curr_el_sp0_fiq_el2
    .balign 0x80
    CALL_WITH_CONTEXT curr_el_sp0_serror_el2

    // from the current EL using the current SP
    .balign 0x80
    CALL_WITH_CONTEXT curr_el_spx_sync_el2
    .balign 0x80
    CALL_WITH_CONTEXT curr_el_spx_irq_el2
    .balign 0x80
    CALL_WITH_CONTEXT curr_el_spx_fiq_el2
    .balign 0x80
    CALL_WITH_CONTEXT curr_el_spx_serror_el2

    // from lower EL (AArch64)
    .balign 0x80
    CALL_WITH_CONTEXT lower_el_aarch64_sync_el2
    .balign 0x80
    CALL_WITH_CONTEXT lower_el_aarch64_irq_el2
    .balign 0x80
    CALL_WITH_CONTEXT lower_el_aarch64_fiq_el2
    .balign 0x80
    CALL_WITH_CONTEXT lower_el_aarch64_serror_el2

    // from lower EL (AArch32)
    .balign 0x80
    CALL_WITH_CONTEXT lower_el_aarch32_sync_el2
    .balign 0x80
    CALL_WITH_CONTEXT lower_el_aarch32_irq_el2
    .balign 0x80
    CALL_WITH_CONTEXT lower_el_aarch32_fiq_el2
    .balign 0x80
    CALL_WITH_CONTEXT lower_el_aarch32_serror_el2
