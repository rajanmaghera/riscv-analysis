main:
    addi    sp, sp, -8  # sp -= 8

    li      a0, 10      # a0 <- 10
    li      a1, 20      # a1 <- 20
    sw      a0, 4(sp)   # sp[4] <- a0
    sw      a1, 2(sp)   # sp[2] <- a1
    # not possible here

    addi    sp, sp, 8   # sp += 8

    # Call exit procedure
    li      a7, 10
    ecall
