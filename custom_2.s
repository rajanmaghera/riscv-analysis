main:
    addi    sp, sp, -8  # sp -= 8

    li      a0, 10      # a0 <- 10
    sw      a0, 4(sp)   # sp[4] <- a0

    li      a1, 20      # a1 <- 20
    addi    sp, sp, 8   # sp += 8
    sw      a1, -8(sp)  # sp[-8] <- a1
    # optimal: str a1, a0, -8(sp)

    # Call exit procedure
    li      a7, 10
    ecall
