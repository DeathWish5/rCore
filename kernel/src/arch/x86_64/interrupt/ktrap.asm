.section .text
.global __kalltraps
.intel_syntax noprefix

__kalltraps:
    push rax
    push rcx
    mov rcx, rsp
    mov rax, 0xFFFF000000000000
    and rcx, rax
    cmp rcx, rax
    je routine
    mov rcx, rsp
    add rcx, 72
    swapgs
    mov rsp, gs:[4]
    swapgs
    push [rcx-8]
    push [rcx-16]
    push [rcx-24]
    push [rcx-32]
    push [rcx-40]
    push [rcx-48]
    push [rcx-56]
    push [rcx-64]
    push [rcx-72]
routine:
    push rdx
    push rdi
    push rsi
    push r8
    push r9
    push r10
    push r11

    push rbx
    push rbp
    push r12
    push r13
    push r14
    push r15

    # push fs.base
    xor rax, rax
    mov ecx, 0xC0000100
    rdmsr # msr[ecx] => edx:eax
    shl rdx, 32
    or rdx, rax
    push rdx

    # save fp registers
    # align to 16 byte boundary
    sub rsp, 512
    mov rax, rsp
    and rax, 0xFFFFFFFFFFFFFFF0
    # fxsave (rax)
    .byte 0x0f
    .byte 0xae
    .byte 0x00
    mov rcx, rsp
    sub rcx, rax
    # push fp state offset
    sub rsp, 16
    push rcx

    mov rdi, rsp
    call rust_trap

.global ktrap_ret
ktrap_ret:

    mov rdi, rsp
    call set_return_rsp

    # pop fp state offset
    pop rcx
    cmp rcx, 16 # only 0-15 are valid
    jge kskip_fxrstor
    mov rax, rsp
    add rax, 16
    sub rax, rcx
    # fxrstor (rax)
    .byte 0x0f
    .byte 0xae
    .byte 0x08
kskip_fxrstor:
    add rsp, 16+512

    # pop fs.base
    pop rax
    mov rdx, rax
    shr rdx, 32
    mov ecx, 0xC0000100
    wrmsr # msr[ecx] <= edx:eax

    pop r15
    pop r14
    pop r13
    pop r12
    pop rbp
    pop rbx

    pop r11
    pop r10
    pop r9
    pop r8
    pop rsi
    pop rdi
    pop rdx
    pop rcx
    pop rax

    # pop trap_num, error_code
    add rsp, 16

    iretq