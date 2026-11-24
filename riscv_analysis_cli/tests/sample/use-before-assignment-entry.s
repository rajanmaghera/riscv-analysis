main:
    jal fn
    li a7, 10
    ecall

fn:
    add a0, a0, t0
    ret
