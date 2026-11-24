main:
    jal fn
    li a7, 10
    ecall

fn:
    sw ra, -4(sp)
    jal leaf
    lw ra, -4(sp)
    ret

leaf:
    addi a0, a0, 1
    ret
