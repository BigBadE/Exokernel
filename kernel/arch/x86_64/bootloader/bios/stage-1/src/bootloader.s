.section .boot, "awx"
.global _start
.code16

_start:
    # Zero registers in case there's any junk data
    xor ax, ax
    mov es, ax
    mov ss, ax

    # Setup Stack
    mov sp, 0x7c00

    # Clear direction flag
    cld
rust:
    push dx
    call first_stage
