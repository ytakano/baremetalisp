SECTIONS
{
/*    . = 0x80000;*/
    . = 0;
    .init : { KEEP(*(.init)) }
    .text : { *(.text .text.* .gnu.linkonce.t*) }
    .rodata : { *(.rodata .rodata.* .gnu.linkonce.r*) }
    PROVIDE(_data = .);
    .data : { *(.data .data.* .gnu.linkonce.d*) }
    .bss (NOLOAD) : {
        . = ALIGN(1024 * 64);
        __bss_start = .;
        __mng_pages    = .;
        __tt_el3_start = __mng_pages + 1024 * 64;
        __tt_el3_end   = __tt_el3_start + 1024 * 64 * 10;
        __tt_el2_start = __tt_el3_end;
        __tt_el2_end   = __tt_el2_start + 1024 * 64 * 10;
        __tt_el1_start = __tt_el2_end;
        __tt_el1_end   = __tt_el1_start + 1024 * 64 * 10;
        *(.bss .bss.*)
        *(COMMON)
        __bss_end = .;
    }
    .stack (NOLOAD) : {
        . = ALIGN(1024 * 64);
        /* 8MiB space for EL3 */
        __stack_el3_end   = .;
        __stack_el3_start = __stack_el3_end + 1024 * 1024 * 8;
        /* 8MiB space for EL2 */
        __stack_el2_end   = __stack_el3_start;
        __stack_el2_start = __stack_el2_end + 1024 * 1024 * 8;
        /* 8MiB space for EL1 */
        __stack_el1_end   = __stack_el2_start;
        __stack_el1_start = __stack_el1_end + 1024 * 1024 * 8;
    }
    _end = __stack_el1_start;

   /DISCARD/ : { *(.comment) *(.gnu*) *(.note*) *(.eh_frame*) }
}
__bss_size = (__bss_end - __bss_start) >> 3;