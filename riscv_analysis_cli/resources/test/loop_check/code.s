main:
	li a0, 10
	li a1, 20 # unused
	li a2, 30 # unused
	jal func1
	li a7, 1
	ecall
	li a7, 10
	ecall


# args: a0, ret: a0
func1:
	addi sp, sp, -4
	sw s0, (sp)
	li s0, 32

	L1:
	beq zero, s0, L2

	addi sp, sp, -4
	sw s1, (sp)
	li s1, 64
	li s2, 39 # BAD --> overwriting
	add s1, s1, s0
	add s1, s1, a0 # Unused value
	lw s1, (sp)
	addi sp, sp, 4
	addi s0, s0, -1
	j L1
	
	L2: 
	mv a0, s0
	lw s0, (sp)
	addi sp, sp, 4
	ret


	