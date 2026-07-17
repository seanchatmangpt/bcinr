	.build_version macos, 11, 0
	.section	__TEXT,__text,regular,pure_instructions
	.globl	_saturating_div
	.p2align	2
_saturating_div:
	.cfi_startproc
	cmp	w1, #0
	cset	w8, eq
	orr	w9, w1, w8
	ubfiz	x10, x0, #16, #32
	udiv	x9, x10, x9
	tst	x9, #0xffff00000000
	csinc	w8, w8, wzr, eq
	cmp	w8, #0
	csinv	w0, w9, wzr, eq
	ret
	.cfi_endproc

.subsections_via_symbols
