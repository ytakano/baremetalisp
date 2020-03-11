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
        . = ALIGN(16);
        __bss_start = .;
        *(.bss .bss.*)
        *(COMMON)
        __bss_end = .;
    }
    .stack : {
        . = ALIGN(1024 * 64);
        __stack_el3_end = .;
        /* 8MiB space for EL3 */
        __stack_el3_start = __stack_el3_end + 1024 * 1024 * 8;
        /* 8MiB space for EL2 */
        __stack_el2_end = .;
        __stack_el2_start = __stack_el2_end + 1024 * 1024 * 8;
    }
    _end = .;

   /DISCARD/ : { *(.comment) *(.gnu*) *(.note*) *(.eh_frame*) }
}
__bss_size = (__bss_end - __bss_start)>>3;