main:
    jal fn
    jal leaf
    li a7, 10
    ecall

fn:
    addi sp, sp, -8
    sw ra, 4(sp)
    li t1, 5
    la t0, leaf
    li a0, 1
    jalr ra, 0(t0)
    add a0, a0, t1
    lw ra, 4(sp)
    addi sp, sp, 8
    ret

leaf:
    addi a0, a0, 1
    ret
