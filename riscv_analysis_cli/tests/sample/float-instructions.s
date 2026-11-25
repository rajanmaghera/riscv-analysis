# Floating-point instructions are tracked for their effect on integer
# registers and memory. fcvt.d.w reads t1 before it is assigned, and
# fcvt.w.d defines a0 before it is printed.
start:
	la	t0, values
	fld	ft0, 0(t0)
	li	a0, 7
	fcvt.d.w	ft1, a0
	fmadd.d	ft1, ft1, ft0, ft0
	fsd	ft1, 8(t0)
	fld	fa0, 8(t0)
	fcvt.d.w	ft2, t1
	fadd.d	fa0, fa0, ft2
	fcvt.w.d	a0, fa0, rtz
	li	a7, 1
	ecall
	li	a7, 10
	ecall

	.data
values:
	.word	0, 0x3ff00000, 0, 0
