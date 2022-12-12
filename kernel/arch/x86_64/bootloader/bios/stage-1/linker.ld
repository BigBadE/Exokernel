ENTRY(_start)

SECTIONS {
    /* End of BIOS section */
    _stack_start = 0x500;
    _stack_end = 0x1500;

    . = 0x1500;
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

    . = 0x7c00 + 446;
    _partition_table = .;
    .partition_table :
    {
        /* partition table entry 0 */
        QUAD(0)
        QUAD(0)
        /* partition table entry 1 */
        QUAD(0)
        QUAD(0)
        /* partition table entry 2 */
        QUAD(0)
        QUAD(0)
        /* partition table entry 3 */
        QUAD(0)
        QUAD(0)
    }

    . = 0x7c00 + 510;

    .magic_number :
    {
        SHORT(0xaa55)       /* magic number for bootable disk */
    }

    _second_stage_start = .;
}