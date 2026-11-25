# Metadata directives are skipped without diagnostics.
	.attribute	4, 16
	.attribute	5, "rv32i2p1_m2p0"
	.file	"metadata.s"
	.text
	.p2align	2
start:
	la	a0, table
	lw	a0, 0(a0)
	li	a7, 93
	ecall
.Lend:
	.size	start, .Lend-start
	.data
	.p2align	3, 0x0
table:
	.quad	0x0000000100000002
	.zero	8
	.size	table, 16
	.ident	"hand written"
	.addrsig
	.addrsig_sym table
