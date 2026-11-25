main:
    jal fn
    li a7, 1
    ecall
    li a7, 10
    ecall

fn:
    li s0, 1
    li a0, 5
    ret
