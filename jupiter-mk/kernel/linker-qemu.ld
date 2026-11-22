OUTPUT_ARCH(riscv)
ENTRY(_start)

BASE_ADDRESS = 0x80200000;

SECTIONS
{
    . = BASE_ADDRESS;
    skernel = .;

    .text : {
        KEEP(*(.text.entry))
        *(.text .text.*)
    }
    . = ALIGN(4096);
    etext = .;

    .rodata : {
        *(.rodata .rodata.*)
    }
    . = ALIGN(4096);
    erodata = .;

    .data : {
        *(.data .data.*)
        *(.sdata .sdata.*)
    }
    . = ALIGN(4096);
    edata = .;

    .bss : {
        sbss = .;
        *(.bss.stack)
        *(.bss .bss.*)
        *(COMMON)
        *(.sbss .sbss.*)
        . = ALIGN(8);
        ebss = .;
    }
    . = ALIGN(4096);
    ekernel = .;

    /DISCARD/ : {
        *(.eh_frame)
        *(.eh_frame_hdr)
        *(.comment)
    }
}
