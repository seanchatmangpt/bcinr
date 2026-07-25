Based on the inspection of `tools/bcinr-cheat-scanner/src/main.rs` (lines 545-570), the cheat scanner detects `CHEAT-021` (Rejection state drift) by verifying that specific test files explicitly include a rejection invariance check. 

Here is exactly how the detection logic works:

### 1. Target File Identification
- The file path must contain `/tests/`.
- The file path must contain `case_studies.rs`.

### 2. Exclusion of Nested/Fixture Files
- The file must **not** be nested in any subdirectories under the `tests/` directory. 
- The scanner splits the path at `/tests/` and ensures the remaining portion of the path does not contain any additional slashes (`/`). 
- This deliberately excludes inert baseline/fixture files (e.g., `tests/fixtures/pre_migration/case_studies.rs`) because Cargo does not treat deeply nested files under `tests/` as live integration test binaries, meaning they cannot contain executable `#[test]` blocks.

### 3. Content Verification
- For any target file that meets the path criteria, its source code is scanned for the exact string `"test_rejection_invariance"`.
- If this string is **missing**, it triggers the `CHEAT-021` violation.

### Violation Message
When triggered, the scanner records the following finding:
```
CHEAT[CHEAT-021]: <file_path> — rejection state drift: case studies missing test_rejection_invariance check
```

**Purpose:** 
This rule enforces the architectural requirement that all live integration case studies explicitly prove that state variables remain bit-for-bit unchanged when an operation is rejected by the branchless runtime.
