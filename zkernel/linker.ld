ENTRY(kernel_main)

SECTIONS {
    /* Kernel will be loaded at 1MB physical address */
    . = 1M;

    .kernel_header : {
        KEEP(*(.kernel_header))
    }

    .text : {
        *(.text .text.*)
    }

    .rodata : {
        *(.rodata .rodata.*)
    }

    .data : {
        *(.data .data.*)
    }

    .bss : {
        *(.bss .bss.*)
    }
}