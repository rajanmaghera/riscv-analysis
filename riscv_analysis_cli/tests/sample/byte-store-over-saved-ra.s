main:
    jal fn
    li a7, 10
    ecall

fn:
    addi sp, sp, -16
    sw ra, 12(sp)
    sb zero, 13(sp)
    lw ra, 12(sp)
    addi sp, sp, 16
    ret
