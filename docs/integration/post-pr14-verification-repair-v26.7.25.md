# Post-PR #14 verification and CI repair receipt

## Ground truth

- Base commit: `9ccc5ec308fce6bab6f2d81cdd7034eea77db64c`
- Former PR #14 head: `66a65853d5fc65d50cab7ffa92febabcdd5a477b`
- Verification workflow run: `30186713984`
- Verified source input: `ed8a28a68ecdc3740f12e7031c42260a52b9c9b8`
- Initial broad E2E standing: `42 passed; 18 failed`
- Initial classification: `PARTIAL_ALIVE / BUILD_BROKEN`

## Repairs

- Added the missing `u64_contract!` marker to `select_u32`.
- Made hostile-mutant assertions require isolated feature admission while preserving all eleven mutant rails.
- Replaced machine-local anti-LLM LSP assumptions with explicit typed skips when the sibling repository is not admitted.
- Made formatting fixtures reachable by `cargo fmt` and converted the nonexistent-directory case into a typed failure assertion.
- Added a strict MCP stdio ingress that returns JSON-RPC `-32700` for malformed JSON before forwarding valid frames to rmcp.
- Split permanent CI into command-level receipts and converted exhaustive mutants into an eleven-job matrix.
- Removed all temporary executors, diagnostics, publishers, adapters, scripts, and logs from the final tree.

## Verification

Every required command completed with exit code 0 in GitHub Actions run `30186220116`, including formatting, workspace compilation, Clippy, all CMCA rails, all eleven isolated mutants, PDDL/POWL, the strict MCP malformed-JSON contract, broad E2E, and the full workspace suite.

## Final classification

`ALIVE` for the observed Linux verification ladder. Permanent PR CI is restored in the published source tree.
