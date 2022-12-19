.global _start
.code16

_start:
    # Zero registers in case there's any junk data
    xor ax, ax
    mov es, ax
    mov ss, ax

    # Setup Stack
    mov sp, 0x7c00

    # Disable interrupts
    cli

    # Test A20 line
    mov ds, 0xFFFF
    mov si, 7E0E

    cmp byte [ds:si], 0xAA55
    je enable_a20
    jump finish_a20
enable_a20:

finish_a20:
rust:
    call first_stage
