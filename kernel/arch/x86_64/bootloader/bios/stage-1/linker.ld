ENTRY(_start)

SECTIONS {
    /* End of BIOS section */
    , = 0x500;
    _stack_start = 0x500;
    _stack_end = 0x7c00;

    . = 0x7c00;
    /* Boot record is loaded at 7c00 */
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

    /* Magic number starts at 0x7c00 + 510 */
    . = 0x7c00 + 510;

    /* magic number for bootable disk */
    .magic_number :
    {
        SHORT(0xaa55)
    }

    _second_stage = .;
}