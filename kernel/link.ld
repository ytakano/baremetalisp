SECTIONS
{
/*    . = 0x80000;*/
    . = 0;
    .init : { KEEP(*(.init)) }
    .text : { *(.text .text.* .gnu.linkonce.t*) }
    .rodata : { *(.rodata .rodata.* .gnu.linkonce.r*) }
    PROVIDE(_data = .);
    .data : ALIGN(1024 * 64) {
        __data_start = .;
        *(.data .data.* .gnu.linkonce.d*)
        __data_end = .;
    }
    .bss (NOLOAD) : ALIGN(1024 * 64) {
        __bss_start    = .;
        *(.bss .bss.*)
        *(COMMON)
        __bss_end = .;
    }
    . = ALIGN(1024 * 64);
    __no_cache = .;

    __stack_start = __no_cache + 1024 * 64;
    /* 8MiB space for EL3 or El2 */
    __stack_firm_end   = __stack_start;
    __stack_firm_start = __stack_firm_end + 1024 * 1024 * 8;
    /* 8MiB space for EL1 */
    __stack_el1_end   = __stack_firm_start;
    __stack_el1_start = __stack_el1_end + 1024 * 1024 * 8;
    /* 8MiB space for EL0 */
    __stack_el0_end   = __stack_el1_start;
    __stack_el0_start = __stack_el0_end + 1024 * 1024 * 8;
    __stack_end       = __stack_el0_start;

    __tt_firm_start = __stack_end;
    __tt_firm_end   = __tt_firm_start + 1024 * 64 * 10;
    __tt_el1_ttbr0_start = __tt_firm_end;
    __tt_el1_ttbr0_end   = __tt_el1_ttbr0_start + 1024 * 64 * 10;
    __tt_el1_ttbr1_start = __tt_el1_ttbr0_end;
    __tt_el1_ttbr1_end   = __tt_el1_ttbr1_start + 1024 * 64 * 2;

    __el0_heap_start = __tt_el1_ttbr1_end;
    __el0_heap_end   = 64 * 1024 * 1024;

    _end = __el0_heap_end;

   /DISCARD/ : { *(.comment) *(.gnu*) *(.note*) *(.eh_frame*) }
}
__bss_size = (__bss_end - __bss_start) >> 3;