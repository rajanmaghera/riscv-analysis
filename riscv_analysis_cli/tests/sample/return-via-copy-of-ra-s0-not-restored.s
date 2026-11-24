main:
    jal fn
    li a7, 10
    ecall

fn:
    mv t0, ra
    li s0, 1
    jr t0
