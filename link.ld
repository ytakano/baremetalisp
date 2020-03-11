SECTIONS
{
    . = 0x80000;
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
    .invalid : {
        . = ALIGN(64 * 1024);
        __invalid_start = .;
        . = . + 1026 * 64;
    }
    .stack : {
        . = ALIGN(16);
        __stack_end = .;
        /* 2MiB space */
        __stack_start = __stack_end + 1024 * 1024 * 2;
    }
    _end = .;

   /DISCARD/ : { *(.comment) *(.gnu*) *(.note*) *(.eh_frame*) }
}
__bss_size = (__bss_end - __bss_start)>>3;