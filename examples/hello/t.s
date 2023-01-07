bits 16
cpu 386
org 0x100

    jmp print_msg
msg:
    db "000000", 0x0D, 0x0A, "$"
print_msg:
    mov dx, msg
    mov ah, 0x09
    int 0x21
    mov ax, 0x4C00
    int 0x21
    hlt
