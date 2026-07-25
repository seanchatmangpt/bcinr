### Detection Heuristic for CHEAT-001

The detection logic for CHEAT-001 is implemented in `tools/bcinr-cheat-scanner/src/main.rs`. It operates at the AST (Abstract Syntax Tree) level using the `syn` crate, which allows it to understand the structure of expressions rather than relying purely on naive text matching. 

To avoid false positives on legitimate bitwise logic, the scanner narrows its focus to specific binary operations: **Bitwise XOR (`^`)** and **Subtraction (`-`)**.

During the AST traversal, it looks for three distinct structural patterns:

1. **Direct Self-Cancellation (`A ^ A` or `A - A`)**:
   It converts the AST nodes of the left and right operands into strings (removing all whitespace) and checks if they are identical.

2. **Left-Side Wrapped Cancellation (`A.wrapping_add(B) ^ A` etc.)**:
   It checks if the left operand is a method call to `wrapping_add` or `wrapping_sub`. If it is, it inspects the *receiver* (`A`) of that method. If the stringified receiver exactly matches the right operand, it flags a violation.

3. **Right-Side Wrapped Cancellation (`A ^ A.wrapping_add(B)` etc.)**:
   Similar to the above, it checks if the right operand is a method call to `wrapping_add` or `wrapping_sub`, and whether its stringified receiver perfectly matches the left operand.

#### The "Simple Expression" Constraint
To prevent over-eager matching and false positives on complex method chains, the scanner enforces a constraint on the receiver of the `wrapping_add` / `wrapping_sub` calls. It uses a recursive `is_simple_expr` helper function to guarantee the receiver is structurally basic. 

The receiver must be one of the following:
- A variable/path (`Expr::Path`)
- A field access (`Expr::Field`)
- An array index (`Expr::Index`)
- A literal (`Expr::Lit`)
- A reference to any of the above (`Expr::Reference`)

By confining the check to exact AST string-matches on simple receivers across XOR and Subtraction boundaries, the heuristic successfully catches artificially injected complexity (like `a.wrapping_add(b) ^ a`) without improperly flagging valid bitwise operations (like those found in hashes or bit-manipulation algorithms).
