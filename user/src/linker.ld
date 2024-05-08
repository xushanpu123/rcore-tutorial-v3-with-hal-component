
OUTPUT_ARCH(riscv)
ENTRY(_start)

BASE_ADDRESS = 0x10000;

SECTIONS
{
    . = BASE_ADDRESS;
    . = ALIGN(4K);
    .text ALIGN(4K): {
        *(.text.entry)
        *(.text .text.*)
    }
    . = ALIGN(4K);
    .rodata ALIGN(4K): {
        *(.rodata .rodata.*)
        *(.srodata .srodata.*)
    }
    . = ALIGN(4K);
    .data ALIGN(4K): {
        *(.data .data.*)
        *(.sdata .sdata.*)
    }
    . = ALIGN(4K);
    .got ALIGN(4K): {
        *(.got .got.*)
    }
    . = ALIGN(4K);
    .bss ALIGN(4K): {
        *(.bss .bss.*)
        *(.sbss .sbss.*)
    }
    /DISCARD/ : {
        *(.eh_frame)
        *(.eh_frame_hdr)
        *(.debug*)
        *(.rela*)
        *(.dynamic*)
    }
}
