ENTRY(_start)

SECTIONS {
    /* Start of third stage */
    . = 0x10000;

    .third_stage :
    {
        *(.sthird_stage)
    }
    .text :
    {
        *(.text .text.*)
    }
    .bss : {
        *(.bss .bss.*)
    }
    .rodata : {
        *(.rodata .rodata.*)
    }
    .data : {
        *(.data .data.*)
    }
    .eh_frame : {
        *(.eh_frame .eh_frame.*)
    }
    .eh_frame_hdr : {
        *(.eh_frame_hdr .eh_frame_hdr.*)
    }

    . = ALIGN(512);

    second_end = .;
}