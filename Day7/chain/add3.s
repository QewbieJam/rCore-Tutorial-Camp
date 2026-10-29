	.attribute	4, 16
	.attribute	5, "rv64i2p1_m2p0_a2p1_f2p2_d2p2_c2p0_zicsr2p0_zifencei2p0_zmmul1p0_zaamo1p0_zalrsc1p0_zca1p0_zcd1p0"
	.file	"add3.a63cad8643495cc9-cgu.0"
	.section	.text.add3,"ax",@progbits
	.globl	add3
	.p2align	1
	.type	add3,@function
add3:
	add	a0, a0, a1
	add	a0, a0, a2
	ret
.Lfunc_end0:
	.size	add3, .Lfunc_end0-add3

	.section	.text.main,"ax",@progbits
	.globl	main
	.p2align	1
	.type	main,@function
main:
	li	a0, 6
	tail	print_int
.Lfunc_end1:
	.size	main, .Lfunc_end1-main

	.ident	"rustc version 1.99.1 (58cf8f9c3 2026-10-13)"
	.section	".note.GNU-stack","",@progbits
