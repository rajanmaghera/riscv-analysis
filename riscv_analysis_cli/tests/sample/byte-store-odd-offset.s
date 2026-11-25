main:
    jal fn
    li a7, 10
    ecall

fn:
    addi sp, sp, -16
    sw ra, 12(sp)
    sb a0, 1(sp)
    sh a0, 6(sp)
    lb a0, 1(sp)
    lw ra, 12(sp)
    addi sp, sp, 16
    ret
