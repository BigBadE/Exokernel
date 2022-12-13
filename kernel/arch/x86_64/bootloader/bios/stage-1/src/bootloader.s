.section .boot "awx"
.global _start
.code16

_start:
    //Setup Stack
    mov sp, 0x7c00

rust:
    call main