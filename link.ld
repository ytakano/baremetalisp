SECTIONS
{
    . = 0x80000;
/*    . = 0;*/
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
    .gnu : { KEEP(*(.hash)) }
    .heap : {
        __heap_start = .;
        /* 64MiB heap */
        . = . + 1024 * 1024 * 64;
        . = ALIGN(4096);
        __heap_end = .;
    }
    .stack : {
        __stack_end = .;
        /* 64MiB space */
        __stack_start = __stack_end + (1024*1024) * 64;
    }
    _end = .;

   /DISCARD/ : { *(.comment) *(.note*) *(.eh_frame*) }
}
__bss_size = (__bss_end - __bss_start)>>3;