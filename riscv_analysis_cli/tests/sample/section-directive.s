# .section consumes its whole line and switches segments by name.
	.section	.text.start,"ax",@progbits
start:
	la	a0, message
	li	a7, 4
	ecall
	li	a7, 10
	ecall

	.section	.rodata.str,"aMS",@progbits,1
message:
	.asciz	"hi\n"

	.section	".note.stack","",@progbits
