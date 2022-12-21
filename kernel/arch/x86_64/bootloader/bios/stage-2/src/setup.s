_begin:
    int 0x10
    jmp enable_a20
test_a20:
    # Test A20 line
    mov di, 0xffff
    mov si, 0x7e0e

    # test byte [ds:si], 0xAA55
    ret
    jmp after_a20
enable_a20:
    call test_a20

    # BIOS a20


    call test_a20

    # Keyboard controller a20


    call test_a20

    # Fast a20
    in al, 0x92
    or al, 2
    out 0x92, al

    call test_a20
    call fail
after_a20:
    call second_stage