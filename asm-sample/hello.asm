            global _start


            section .text

            ; x = [rbp-8]
            ; y = [rbp-16]
_start:     
            mov     rbp, rsp
            sub     rsp, 16
            mov     qword [rbp-8], 1
            mov     qword [rbp-16], 0
loop1:      
            ; y += x
            mov     rbx, [rbp-8]
            mov     rax, [rbp-16]
            add     rax, rbx
            mov     [rbp-16], rax

            ; x += 1
            mov     rax, [rbp-8]
            add     rax, 1
            mov     qword [rbp-8], rax

            ; if x > 10
            mov     rax, [rbp-8]
            cmp     rax, 10
            jg      loop2
            jmp     loop1
loop2:      
            mov     rax, [rbp-16]        ; return y
            add     rsp, 16

            ret     0

