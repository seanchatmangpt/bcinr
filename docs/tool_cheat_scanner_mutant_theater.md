Based on an inspection of `/Users/sac/bcinr/tools/bcinr-cheat-scanner/src/main.rs`, here is how `CHEAT-009` (Mutant theater) is detected:

The scanner looks for instances of `mutant` within test files. It flags a `CHEAT-009` violation if the test uses a bare `assert_ne!` to verify the baseline against the mutant without checking for a typed refusal or a named mathematical law violation.

Specifically, if a test file containing `"mutant"` uses `"assert_ne!"`, it MUST also contain at least one of the following pieces of evidence indicating proper typed refusal or invariant tracking:
- `Err(StabilityRefusal::`
- `Err(ObservatoryFlag::`
- `.is_refused()`
- `.numeric_faults()`
- `.faults()`
- `.refusals()`
- `// Named law:`

If `assert_ne!` is present but **none** of the above strings are found in the file, the scanner pushes the following finding:
`CHEAT[CHEAT-009]: <path> — mutant theater: test uses weak assert_ne instead of asserting a typed refusal`
