.global _start
.code16

_start:
    # Setup Stack
    mov sp, 0x7c00

rust:
    call first_stage