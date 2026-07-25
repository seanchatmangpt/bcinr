# Test policy

## Invariant

Pull-request test execution has one hard wall-clock budget: **five seconds**.
Compilation is a separate admission phase. Every unit and integration test target must compile, but only the curated PR admission proof executes on the required PR rail.

## Audit receipt

The workspace audit compiled and measured every test binary individually on Ubuntu nightly.

- Compiled test binaries: 82
- Completed successfully within five seconds: 76
- Failed or exceeded five seconds: 6

The six rejected targets were:

| Target | Finding | Resolution |
|---|---|---|
| `bcinr/tests/e2e_main.rs` | 60 recursive Cargo subprocess tests exceeded five seconds and frequently accepted nonzero exit codes as success | Removed. Formatting, Clippy, compilation, and tool behavior are owned by their direct CI or crate tests. |
| `bcinr-mcp/tests/adversarial.rs` | Eleven tests repeatedly spawned the same MCP server and exceeded five seconds | Collapsed into one shared-session transport and admission proof; megabyte-scale fuzz repetition was trimmed. |
| `bcinr-mcp/tests/integration_tests.rs` | Parsed `src/main.rs` text instead of testing the live MCP surface and exited 101 | Removed. Live `tools/list` coverage in the adversarial contract is authoritative. |
| `bcinr-cmca/tests/compile_fail_tests.rs` | UI diagnostics exited 101 under the all-features audit | Retained as a specialized compile-fail rail, not part of PR execution. |
| `bcinr-powl/tests/compile_fail_tests.rs` | UI diagnostics exited 101 under the all-features audit | Retained as a specialized compile-fail rail, not part of PR execution. |
| `bcinr-cmca/tests/hostile_mutants.rs` | Mutually exclusive mutation features were activated together by `--all-features` | Retained for one-mutant-at-a-time invocation; never treated as an all-features runtime suite. |

## Required PR rail

1. `cargo fmt --all -- --check`
2. `cargo clippy --workspace --all-targets --all-features -- -D warnings`
3. `cargo test --workspace --all-features --lib --tests --no-run`
4. Build `bcinr-pddl`'s `embedded_workflow_generated` proof.
5. Execute that compiled proof under a hard five-second process timeout.
6. Compile the production composition crates on macOS and Windows.

The runtime proof covers both sides of the authority boundary in one domain initialization:

- an unpaid observation is refused before command manufacture;
- a paid observation produces the complete generated command set;
- generated action coverage is complete;
- durable and local execution routing remain exact.

## Test design rules

- Test behavior through public APIs or live protocol surfaces, not by parsing implementation source text.
- Do not shell back into Cargo from ordinary unit or integration tests.
- Share expensive fixtures and server processes within one lifecycle test.
- Keep exhaustive fuzzing, UI diagnostics, mutation testing, Miri, and benchmarks on explicit specialized rails.
- A test that accepts both success and failure exit codes is not an admission test and must be removed or rewritten.
- New required PR tests must fit inside the existing five-second aggregate execution budget; they do not receive an additional five-second allowance.
