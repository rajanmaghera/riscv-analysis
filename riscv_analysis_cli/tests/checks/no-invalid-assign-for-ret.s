main:
    jal     other
    li      ra, 0

other:
    addi    a0, a0, 1   # Error for unused value
    ret                 # There should not be an error here
