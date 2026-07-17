[1m[33mwarning[0m[1m: value assigned to `bit` is never read[0m
  [1m[94m--> [0mcrates/bcinr-logic/src/algorithms/fp_sqrt_u32_q16.rs:26:13
   [1m[94m|[0m
[1m[94m26[0m [1m[94m|[0m             bit >>= 2;
   [1m[94m|[0m             [1m[33m^^^^^^^^^[0m
[1m[94m...[0m
[1m[94m31[0m [1m[94m|[0m     unroll_16!();
   [1m[94m|[0m     [1m[94m------------[0m [1m[94min this macro invocation[0m
   [1m[94m|[0m
   [1m[94m= [0m[1mhelp[0m: maybe it is overwritten before being read?
   [1m[94m= [0m[1mnote[0m: `#[warn(unused_assignments)]` (part of `#[warn(unused)]`) on by default
   [1m[94m= [0m[1mnote[0m: this warning originates in the macro `step` which comes from the expansion of the macro `unroll_16` (in Nightly builds, run with -Z macro-backtrace for more info)

[1m[33mwarning[0m[1m: value assigned to `v` is never read[0m
  [1m[94m--> [0mcrates/bcinr-logic/src/algorithms/fp_sqrt_u32_q16.rs:23:13
   [1m[94m|[0m
[1m[94m23[0m [1m[94m|[0m             v = crate::ct::ct_select_u32(combined, v.wrapping_sub(res + bit), v);
   [1m[94m|[0m             [1m[33m^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^[0m
[1m[94m...[0m
[1m[94m31[0m [1m[94m|[0m     unroll_16!();
   [1m[94m|[0m     [1m[94m------------[0m [1m[94min this macro invocation[0m
   [1m[94m|[0m
   [1m[94m= [0m[1mhelp[0m: maybe it is overwritten before being read?
   [1m[94m= [0m[1mnote[0m: this warning originates in the macro `step` which comes from the expansion of the macro `unroll_16` (in Nightly builds, run with -Z macro-backtrace for more info)

[1m[33mwarning[0m[1m: value assigned to `b` is never read[0m
  [1m[94m--> [0mcrates/bcinr-logic/src/algorithms/gcd_u64_branchless.rs:37:13
   [1m[94m|[0m
[1m[94m37[0m [1m[94m|[0m             b = ct_select_u64(b_was_zero, 0, diff);
   [1m[94m|[0m             [1m[33m^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^[0m
[1m[94m...[0m
[1m[94m51[0m [1m[94m|[0m     unroll_64!();
   [1m[94m|[0m     [1m[94m------------[0m [1m[94min this macro invocation[0m
   [1m[94m|[0m
   [1m[94m= [0m[1mhelp[0m: maybe it is overwritten before being read?
   [1m[94m= [0m[1mnote[0m: this warning originates in the macro `step` which comes from the expansion of the macro `unroll_64` (in Nightly builds, run with -Z macro-backtrace for more info)

