main:
    jal fn
    bnez a0, done
done:
    li a7, 10
    ecall

fn:
    mv t0, ra
    li a0, 5
    jr t0
