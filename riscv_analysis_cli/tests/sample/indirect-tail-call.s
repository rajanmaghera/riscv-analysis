main:
    jal fn
    jal leaf
    li a7, 10
    ecall

fn:
    li a0, 1
    la t0, leaf
    jr t0

leaf:
    addi a0, a0, 1
    ret
