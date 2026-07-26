# CMCA Divan Clippy Diagnostic

- Workflow run: `30189231249`
- Source commit: `40084bbc72330226d6ffe82e100f3bc4b4d76da1`

```text
[1m[92m    Checking[0m bcinr-logic v26.7.25 (/home/runner/work/bcinr/bcinr/crates/bcinr-logic)
[1m[92m    Checking[0m bcinr-mfw-ir v26.7.25 (/home/runner/work/bcinr/bcinr/crates/bcinr-mfw-ir)
[1m[92m    Checking[0m encode_unicode v1.0.1 (/home/runner/work/bcinr/bcinr/crates/encode_unicode_patch)
[1m[92m    Checking[0m bcinr-pddl v26.7.25 (/home/runner/work/bcinr/bcinr/crates/bcinr-pddl)
[1m[92m    Checking[0m prettytable-rs v0.10.0
[1m[92m    Checking[0m bcinr-powl v26.7.25 (/home/runner/work/bcinr/bcinr/crates/bcinr-powl)
[1m[92m    Checking[0m bcinr-api v26.7.25 (/home/runner/work/bcinr/bcinr/crates/bcinr-api)
[1m[92m    Checking[0m bcinr-cmca v26.7.25 (/home/runner/work/bcinr/bcinr/crates/bcinr-cmca)
[1m[92m    Checking[0m bcinr-core v26.7.25 (/home/runner/work/bcinr/bcinr/bcinr-core)
[1m[92m    Checking[0m bcinr-powl-receipt v26.7.25 (/home/runner/work/bcinr/bcinr/crates/bcinr-powl-receipt)
[1m[92m    Checking[0m bcinr-mcp v0.1.0 (/home/runner/work/bcinr/bcinr/crates/bcinr-mcp)
[1m[92m    Checking[0m bcinr-bench v26.7.25 (/home/runner/work/bcinr/bcinr/bcinr-bench)
[1m[91merror[0m[1m: unused `std::result::Result` that must be used[0m
   [1m[94m--> [0mbcinr-bench/benches/cmca_execution_bench.rs:474:9
    [1m[94m|[0m
[1m[94m474[0m [1m[94m|[0m         divan::black_box(observatory_result);
    [1m[94m|[0m         [1m[91m^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^[0m
    [1m[94m|[0m
    [1m[94m= [0m[1mnote[0m: this `Result` may be an `Err` variant, which should be handled
    [1m[94m= [0m[1mnote[0m: `-D unused-must-use` implied by `-D warnings`
    [1m[94m= [0m[1mhelp[0m: to override `-D warnings` add `#[allow(unused_must_use)]`
[1m[96mhelp[0m: use `let _ = ...` to ignore the resulting value
    [1m[94m|[0m
[1m[94m474[0m [1m[94m| [0m        [92mlet _ = [0mdivan::black_box(observatory_result);
    [1m[94m|[0m         [92m+++++++[0m

[1m[91merror[0m: could not compile `bcinr-bench` (bench "cmca_execution_bench") due to 1 previous error
```
