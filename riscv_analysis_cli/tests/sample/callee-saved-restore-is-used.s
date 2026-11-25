# Restoring s0 and sp before returning is required by the calling
# convention, so neither restore is an unused value.
start:
	li	s0, 5
	jal	work
	add	a0, a0, s0
	li	a7, 1
	ecall
	li	a7, 10
	ecall

work:
	addi	sp, sp, -16
	sw	s0, 12(sp)
	li	s0, 3
	addi	a0, s0, 1
	lw	s0, 12(sp)
	addi	sp, sp, 16
	ret
