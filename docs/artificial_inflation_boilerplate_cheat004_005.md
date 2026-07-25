Based on the source code in `bcinr-cheat-scanner/src/main.rs`, the scanner uses purely text-based heuristics (not AST-based) to identify padding, repeated comments, and boilerplate claims.

### `CHEAT-004`: Artificial File Inflation
The scanner uses two heuristics to detect this:
1. **Explicit Padding String Match**: 
   It checks if the entire file content contains the exact substring `"PADDING ENSURING FILE LENGTH REQUIREMENT"`. If found, it triggers the violation `"artificial file-length inflation detected"`.
2. **Consecutive Numbered Padding Block Detection**: 
   It iterates through the file line-by-line looking for consecutive single-line comments (`//`). After stripping the leading slashes and whitespace, it checks if the comment contains the substring `". Line"`. If it encounters **5 or more consecutive lines** matching this pattern, it triggers the `"numbered padding block detected"` violation. Any non-comment line or comment line that doesn't match the pattern resets the counter.

### `CHEAT-005`: Boilerplate Verification Claims
The scanner uses a line-counting heuristic for this:
1. **Mock Hoare-logic Verification Claims**: 
   It iterates through the file and counts lines that contain **both** of the following substrings:
   - `"Hoare-logic Verification Line"`
   - `"Branchless path is the unique solution to the state constraints of"`
   If it finds **5 or more lines** matching this criteria anywhere in the file (they *do not* need to be consecutive), it triggers the `"mock Hoare-logic verification claims detected"` violation.
