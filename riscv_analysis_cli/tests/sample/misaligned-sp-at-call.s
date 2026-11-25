main:
    jal fn
    li a7, 10
    ecall

fn:
    addi sp, sp, -16
    sw ra, 12(sp)
    addi sp, sp, -2
    jal leaf
    addi sp, sp, 2
    lw ra, 12(sp)
    addi sp, sp, 16
    ret

leaf:
    addi a0, a0, 1
    ret
