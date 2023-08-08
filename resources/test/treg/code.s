# heesss 

#     dj
    li a0, 1
    li a1, 2
    li t2, 2
    jal foo
    li a0, 2321
    add    t3 , ,,,  t2,    t4
    add t0, a0, a1
    jal bar
    li a7, 10
    ecall

bar:
    addi sp, sp, -4
    sw ra, (sp)
    sw s0, 4(sp)
    li s0, 2
    jal foo
    lw ra, (sp)
    lw s0, 4(sp)
    addi sp, sp, 4

    ret

foo:
    add a0, a0, a1
    ret

