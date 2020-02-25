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

.L4:
    bl      entry
    b       _start
