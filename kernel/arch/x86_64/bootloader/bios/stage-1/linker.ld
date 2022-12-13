ENTRY(_start)

SECTIONS {
    /* End of BIOS section */
    _stack_start = 0x500;
    _stack_end = 0x1500;

    . = 0x7c00;
    _mbr_start = .;
    .boot :
    {
        *(.boot .boot.*)
    }
    .text :
    {
        *(.text .text.*)
    }
    .rodata :
    {
        *(.rodata .rodata.*)
    }
    .data :
    {
        *(.rodata .rodata.*)
        *(.data .data.*)
        *(.got .got.*)
    }
    _mbr_end = .;

    _partition_table = 0x7c00 + 446;
    /* */
    . = 0x7c00 + 510;

    /* magic number for bootable disk */
    .magic_number :
    {
        SHORT(0xaa55)
    }

    _second_stage_start = .;
}