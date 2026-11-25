# puts_line is defined elsewhere. With --undefined-calls-are-external it is
# an external function that takes its arguments in a0-a7.
start:
	la	a0, greeting
	call	puts_line
	li	a7, 10
	ecall

	.data
greeting:
	.asciz	"hello"
