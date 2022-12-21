# Bootloader

Inspired by rust-osdev/bootloader, provides BIOS and UEFI bootloaders.

# Memory Layout

0x0 - 0x4FF: BIOS
0x500 - 0x7c00: Stack
0x7c01 - 0x7DB8: Stage 1
0x7DB9 - 0x7DFD: Partition table
0x7DFE & 0x7DFF: Magic number
0x7E00 - 0x100000: Stage 2
