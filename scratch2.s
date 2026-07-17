	.build_version macos, 11, 0
	.section	__TEXT,__text,regular,pure_instructions
	.globl	_check
	.p2align	2
_check:
	.cfi_startproc
	cmp	w0, #0
	fmov	s0, #-1.00000000
	fmov	s1, #1.00000000
	fcsel	s0, s1, s0, ne
	ret
	.cfi_endproc

.subsections_via_symbols
