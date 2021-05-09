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

 /* Copyright (c) 2015, Linaro Limited */

.section .init, "x"
.global _start

#define BIT32(nr) (1 << (nr))

#define SCTLR_M    BIT32(0)
#define SCTLR_A    BIT32(1)
#define SCTLR_C    BIT32(2)
#define SCTLR_SA   BIT32(3)
#define SCTLR_I    BIT32(12)
#define SCTLR_WXN  BIT32(19)
#define SCTLR_SPAN BIT32(23)

#define DAIFBIT_FIQ  BIT32(0)
#define DAIFBIT_IRQ  BIT32(1)
#define DAIFBIT_ABT  BIT32(2)
#define DAIFBIT_DBG  BIT32(3)
#define DAIFBIT_ALL  (DAIFBIT_FIQ | DAIFBIT_IRQ | DAIFBIT_ABT | DAIFBIT_DBG)

_start:
    mov     x19, x0         /* Save pagable part address */
    mov     x20, x2         /* Save DT address */

    // set exception vector
    ldr     x0, =exception_vector_el1
    msr     vbar_el1, x0

    // Select SP_EL0
    msr     spsel, #0

    // Set up SCTLR_EL1
    isb
    mrs     x0, sctlr_el1
    orr     x0, x0, #SCTLR_I    // Instruction access Cacheability control
    orr     x0, x0, #SCTLR_SA   // SP Alignment check enable
    orr     x0, x0, #SCTLR_SPAN // Set Priviledged Access Never when ARMv8.1-PAN is implemented, otherwise RES1
    msr     sctlr_el1, x0
    isb

    // set stack before _start
    ldr     x1, =__stack_el1_start
    mrs     x2, mpidr_el1 // read cpu id
    and     x2, x2, #0xFF
    mov     x4, #(STACKSIZE)
    mul     x3, x2, x4
    sub     x1, x1, x3
    mov     sp, x1 // set sp_el1

    // Select SP_EL0
    msr     spsel, #0
    mov     sp, x1 // set sp_el0

    cbnz    x2, .L4

    // if cpu id == 0
.L2:
    // clear bss
    ldr     x1, =__bss_start
    ldr     w2, =__bss_size
.L3:
    cbz     w2, .L4
    str     xzr, [x1], #8
    sub     w2, w2, #1
    cbnz    w2, .L3

.L4:
    // Enable aborts now that we can receive exceptions
    msr     daifclr, #DAIFBIT_ABT

    bl      entry
.L5:
    wfe
    b       .L5
