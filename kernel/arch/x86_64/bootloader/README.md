# Bootloader

Inspired by rust-osdev/bootloader, provides BIOS and UEFI bootloaders.

# Memory Layout

- 0x0 - 0x4FF: BIOS
- 0x500 - 0x7C00: Stack
- 0x7C01 - 0x7DB8: Stage 1
- 0x7DB9 - 0x7DFD: Partition table
- 0x7DFE - 0x7DFF: Magic number
- 0x7E00 - 0xFFFF: Stage 2
- 0x10000 - 0x2FFFF: Stage 3
- 0x30000 - 0x4FFFF: Stage 4
- 0x80000 - 0x100000: BIOS reserved