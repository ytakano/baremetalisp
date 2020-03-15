/*
 * MIT License
 *
 * Copyright (c) 2020 Yuuki Takano <ytakanoster@gmail.com>
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 */

/*
 * Copyright (c) 2016-2019 Raspberry Pi (Trading) Ltd.
 * Copyright (c) 2016 Stephen Warren <swarren@wwwdotorg.org>
 * All rights reserved.
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions are met:
 * * Redistributions of source code must retain the above copyright notice,
 *   this list of conditions and the following disclaimer.
 * * Redistributions in binary form must reproduce the above copyright notice,
 *   this list of conditions and the following disclaimer in the documentation
 *   and/or other materials provided with the distribution.
 * * Neither the name of the copyright holder nor the names of its contributors
 *   may be used to endorse or promote products derived from this software
 *   without specific prior written permission.
 *
 * THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS"
 * AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
 * IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE
 * ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE
 * LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR
 * CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF
 * SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS
 * INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN
 * CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE)
 * ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE
 * POSSIBILITY OF SUCH DAMAGE.
 */

.section .init, "x"
.global _start

// #define NOEL3

#ifdef raspi4
#define BCM2711 1
#define GIC     1
#endif

#define BIT(x) (1 << (x))

#if BCM2711
#define LOCAL_CONTROL       0xff800000
#define LOCAL_PRESCALER     0xff800008
#else
#define LOCAL_CONTROL       0x40000000
#define LOCAL_PRESCALER     0x40000008
#endif
#define GIC_DISTB           0xff841000
#define GIC_CPUB            0xff842000

#if BCM2711
#define OSC_FREQ            54000000
#else
#define OSC_FREQ            19200000
#endif

#define SCR_RW              BIT(10)
#define SCR_HCE             BIT(8)
#define SCR_SMD             BIT(7)
#define SCR_RES1_5          BIT(5)
#define SCR_RES1_4          BIT(4)
#define SCR_NS              BIT(0)
#define SCR_VAL \
    (SCR_RW | SCR_HCE | SCR_RES1_5 | SCR_RES1_4)

#define CPUECTLR_EL1        S3_1_C15_C2_1
#define CPUECTLR_EL1_SMPEN  BIT(6)

#define SPSR_EL3_D          BIT(9)
#define SPSR_EL3_A          BIT(8)
#define SPSR_EL3_I          BIT(7)
#define SPSR_EL3_F          BIT(6)
#define SPSR_EL3_MODE_el3H  9
#define SPSR_EL3_VAL \
    (SPSR_EL3_D | SPSR_EL3_A | SPSR_EL3_I | SPSR_EL3_F | SPSR_EL3_MODE_el3H)

#define L2CTLR_EL1          S3_1_C11_C0_2


#define GICC_CTRLR          0x0
#define GICC_PMR            0x4
#define IT_NR               0x8 // Number of interrupt enable registers (256 total irqs)
#define GICD_CTRLR          0x0
#define GICD_IGROUPR        0x80

_start:
#ifndef NOEL3
#ifdef raspi4
    /*
     * LOCAL_CONTROL:
     * Bit 9 clear: Increment by 1 (vs. 2).
     * Bit 8 clear: Timer source is 19.2MHz crystal (vs. APB).
     */
    mov x0, LOCAL_CONTROL
    str wzr, [x0]
    /* LOCAL_PRESCALER; divide-by (0x80000000 / register_val) == 1 */
    mov w1, 0x80000000
    str w1, [x0, #(LOCAL_PRESCALER - LOCAL_CONTROL)]

    /* Set L2 read/write cache latency to 3 */
    mrs x0, L2CTLR_EL1
    mov x1, #0x22
    orr x0, x0, x1
    msr L2CTLR_EL1, x0

    /* Set up CNTFRQ_EL0 */
    ldr x0, =OSC_FREQ
    msr CNTFRQ_EL0, x0

    /* Set up CNTVOFF_el2 */
    msr CNTVOFF_el2, xzr

    /* Enable FP/SIMD */
    /* All set bits below are res1; bit 10 (TFP) is set to 0 */
    mov x0, #0x33ff
    msr CPTR_EL3, x0

    /* Set up SCR */
    mov x0, #SCR_VAL
    msr SCR_EL3, x0

    /* Set SMPEN */
    mov x0, #CPUECTLR_EL1_SMPEN
    msr CPUECTLR_EL1, x0

#ifdef GIC
    bl      setup_gic
#endif
#endif // NOEL3

#endif // raspi4

    // disable all interrupt (daif at bits 9..6)
    msr     DAIFSet, #0x0f

    // read cpu id, stop slave cores
    mrs     x1, mpidr_el1
    and     x1, x1, #0xFF
    cbz     x1, .L2

    // if cpu id > 0 then stop
.L1:
    wfe
    b       .L1

    // if cpu id == 0
.L2:
    // set stack before _start
    ldr     x1, =__stack_el3_start
    mov     sp, x1

    // clear bss
    ldr     x1, =__bss_start
    ldr     w2, =__bss_size
.L3:
    cbz     w2, .L4
    str     xzr, [x1], #8
    sub     w2, w2, #1
    cbnz    w2, .L3

.L4:
    // set exception vector
    ldr     x1, =exception_vector_el1
    mov     x2, #(0xFFFFFC << 40) // 64KiB page tables, 22 MSB bits are 1
    add     x1, x1, x2
    msr     vbar_el1, x1

    ldr     x1, =exception_vector_el2
    msr     vbar_el2, x1

#if defined(NOEL3) || defined(raspi3) || defined(raspi2)
    mrs     x1, hcr_el2
    orr     x1, x1, #0b111000 // AMO, IMO, FMO
    msr     hcr_el2, x1
#else
    ldr     x1, =exception_vector_el3
    msr     vbar_el3, x1

    mrs     x1, scr_el3
    orr     x1, x1, #0b1110 // EA, FIQ, IRQ
    msr     scr_el3, x1
#endif
    msr     DAIFClr, #0x0f // enable all interrupt

    bl      entry
.L5:
    wfe
    b       .L5

#ifdef GIC

// Called from secure mode - set all interrupts to group 1 and enable.
setup_gic:
    mrs     x0, MPIDR_EL1
    ldr     x2, =GIC_DISTB
    tst     x0, #0x3
    b.eq    2f // primary core

    mov     w0, #3 // Enable group 0 and 1 IRQs from distributor
    str     w0, [x2, #GICD_CTRLR]
2:
    add     x1, x2, #(GIC_CPUB - GIC_DISTB)
    mov     w0, #0x1e7
    str     w0, [x1, #GICC_CTRLR] // Enable group 1 IRQs from CPU interface
    mov     w0, #0xff
    str     w0, [x1, #GICC_PMR] // priority mask
    add     x2, x2, #GICD_IGROUPR
    mov     x0, #(IT_NR * 4)
    mov     w1, #~0 // group 1 all the things
3:
    subs    x0, x0, #4
    str     w1, [x2, x0]
    b.ne    3b
    ret

#endif

.macro CALL_WITH_CONTEXT handler elr_reg spsr_reg
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

    // Add the exception link register and the saved program status.
    mrs     x1, \elr_reg
    mrs     x2, \spsr_reg

    stp     lr,  x1,  [sp, #16 * 15]
    str     w2,       [sp, #16 * 16]

    // x0 is the first argument for the function called through `\handler`.
    mov     x0,  sp

    // Call `\handler`.
    bl      \handler

    ldr     w19,      [sp, #16 * 16]
    ldp     lr,  x20, [sp, #16 * 15]

    msr     \spsr_reg, x19
    msr     \elr_reg,  x20

    // After returning from exception handling code, replay the saved context and return via `eret`.
    b       exception_restore_context
.endm

//--------------------------------------------------------------------------------------------------
// Helper functions
//--------------------------------------------------------------------------------------------------
exception_restore_context:
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

    b       .L5 // hung-up temporary

    eret

    .balign 0x800
exception_vector_el3:
    // from the current EL using the current SP0
    CALL_WITH_CONTEXT curr_el_sp0_sync_el3 ELR_EL3 SPSR_EL3
    .balign 0x80
    CALL_WITH_CONTEXT curr_el_sp0_irq_el3 ELR_EL3 SPSR_EL3
    .balign 0x80
    CALL_WITH_CONTEXT curr_el_sp0_fiq_el3 ELR_EL3 SPSR_EL3
    .balign 0x80
    CALL_WITH_CONTEXT curr_el_sp0_serror_el3 ELR_EL3 SPSR_EL3

    // from the current EL using the current SP
    .balign 0x80
    CALL_WITH_CONTEXT curr_el_spx_sync_el3 ELR_EL3 SPSR_EL3
    .balign 0x80
    CALL_WITH_CONTEXT curr_el_spx_irq_el3 ELR_EL3 SPSR_EL3
    .balign 0x80
    CALL_WITH_CONTEXT curr_el_spx_fiq_el3 ELR_EL3 SPSR_EL3
    .balign 0x80
    CALL_WITH_CONTEXT curr_el_spx_serror_el3 ELR_EL3 SPSR_EL3

    // from lower EL (AArch64)
    .balign 0x80
    CALL_WITH_CONTEXT lower_el_aarch64_sync_el3 ELR_EL3 SPSR_EL3
    .balign 0x80
    CALL_WITH_CONTEXT lower_el_aarch64_irq_el3 ELR_EL3 SPSR_EL3
    .balign 0x80
    CALL_WITH_CONTEXT lower_el_aarch64_fiq_el3 ELR_EL3 SPSR_EL3
    .balign 0x80
    CALL_WITH_CONTEXT lower_el_aarch64_serror_el3 ELR_EL3 SPSR_EL3

    // from lower EL (AArch32)
    .balign 0x80
    CALL_WITH_CONTEXT lower_el_aarch32_sync_el3 ELR_EL3 SPSR_EL3
    .balign 0x80
    CALL_WITH_CONTEXT lower_el_aarch32_irq_el3 ELR_EL3 SPSR_EL3
    .balign 0x80
    CALL_WITH_CONTEXT lower_el_aarch32_fiq_el3 ELR_EL3 SPSR_EL3
    .balign 0x80
    CALL_WITH_CONTEXT lower_el_aarch32_serror_el3 ELR_EL3 SPSR_EL3

    .balign 0x800
exception_vector_el2:
    // from the current EL using the current SP0
    CALL_WITH_CONTEXT curr_el_sp0_sync_el2 ELR_EL2 SPSR_EL2
    .balign 0x80
    CALL_WITH_CONTEXT curr_el_sp0_irq_el2 ELR_EL2 SPSR_EL2
    .balign 0x80
    CALL_WITH_CONTEXT curr_el_sp0_fiq_el2 ELR_EL2 SPSR_EL2
    .balign 0x80
    CALL_WITH_CONTEXT curr_el_sp0_serror_el2 ELR_EL2 SPSR_EL2

    // from the current EL using the current SP
    .balign 0x80
    CALL_WITH_CONTEXT curr_el_spx_sync_el2 ELR_EL2 SPSR_EL2
    .balign 0x80
    CALL_WITH_CONTEXT curr_el_spx_irq_el2 ELR_EL2 SPSR_EL2
    .balign 0x80
    CALL_WITH_CONTEXT curr_el_spx_fiq_el2 ELR_EL2 SPSR_EL2
    .balign 0x80
    CALL_WITH_CONTEXT curr_el_spx_serror_el2 ELR_EL2 SPSR_EL2

    // from lower EL (AArch64)
    .balign 0x80
    CALL_WITH_CONTEXT lower_el_aarch64_sync_el2 ELR_EL2 SPSR_EL2
    .balign 0x80
    CALL_WITH_CONTEXT lower_el_aarch64_irq_el2 ELR_EL2 SPSR_EL2
    .balign 0x80
    CALL_WITH_CONTEXT lower_el_aarch64_fiq_el2 ELR_EL2 SPSR_EL2
    .balign 0x80
    CALL_WITH_CONTEXT lower_el_aarch64_serror_el2 ELR_EL2 SPSR_EL2

    // from lower EL (AArch32)
    .balign 0x80
    CALL_WITH_CONTEXT lower_el_aarch32_sync_el2 ELR_EL2 SPSR_EL2
    .balign 0x80
    CALL_WITH_CONTEXT lower_el_aarch32_irq_el2 ELR_EL2 SPSR_EL2
    .balign 0x80
    CALL_WITH_CONTEXT lower_el_aarch32_fiq_el2 ELR_EL2 SPSR_EL2
    .balign 0x80
    CALL_WITH_CONTEXT lower_el_aarch32_serror_el2 ELR_EL2 SPSR_EL2

    .balign 0x800
exception_vector_el1:
    // from the current EL using the current SP0
    CALL_WITH_CONTEXT curr_el_sp0_sync_el1 ELR_EL1 SPSR_EL1
    .balign 0x80
    CALL_WITH_CONTEXT curr_el_sp0_irq_el1 ELR_EL1 SPSR_EL1
    .balign 0x80
    CALL_WITH_CONTEXT curr_el_sp0_fiq_el1 ELR_EL1 SPSR_EL1
    .balign 0x80
    CALL_WITH_CONTEXT curr_el_sp0_serror_el1 ELR_EL1 SPSR_EL1

    // from the current EL using the current SP
    .balign 0x80
    CALL_WITH_CONTEXT curr_el_spx_sync_el1 ELR_EL1 SPSR_EL1
    .balign 0x80
    CALL_WITH_CONTEXT curr_el_spx_irq_el1 ELR_EL1 SPSR_EL1
    .balign 0x80
    CALL_WITH_CONTEXT curr_el_spx_fiq_el1 ELR_EL1 SPSR_EL1
    .balign 0x80
    CALL_WITH_CONTEXT curr_el_spx_serror_el1 ELR_EL1 SPSR_EL1

    // from lower EL (AArch64)
    .balign 0x80
    CALL_WITH_CONTEXT lower_el_aarch64_sync_el1 ELR_EL1 SPSR_EL1
    .balign 0x80
    CALL_WITH_CONTEXT lower_el_aarch64_irq_el1 ELR_EL1 SPSR_EL1
    .balign 0x80
    CALL_WITH_CONTEXT lower_el_aarch64_fiq_el1 ELR_EL1 SPSR_EL1
    .balign 0x80
    CALL_WITH_CONTEXT lower_el_aarch64_serror_el1 ELR_EL1 SPSR_EL1

    // from lower EL (AArch32)
    .balign 0x80
    CALL_WITH_CONTEXT lower_el_aarch32_sync_el1 ELR_EL1 SPSR_EL1
    .balign 0x80
    CALL_WITH_CONTEXT lower_el_aarch32_irq_el1 ELR_EL1 SPSR_EL1
    .balign 0x80
    CALL_WITH_CONTEXT lower_el_aarch32_fiq_el1 ELR_EL1 SPSR_EL1
    .balign 0x80
    CALL_WITH_CONTEXT lower_el_aarch32_serror_el1 ELR_EL1 SPSR_EL1

.section .bss

.section .stack