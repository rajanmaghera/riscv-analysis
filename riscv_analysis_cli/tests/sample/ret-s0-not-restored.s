main:
    jal fn
    li a7, 10
    ecall

fn:
    li s0, 1
    ret
