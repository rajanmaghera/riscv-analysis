main:
    jal fn
    li a7, 10
    ecall

fn:
    la t0, target
    jr t0

target:
    li s0, 1
    ret
