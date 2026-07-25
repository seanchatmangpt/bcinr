Here is the documentation on how the cheat scanner detects `CHEAT-001` (Self-canceling operations) in `tools/bcinr-cheat-scanner/src/main.rs`.

### CHEAT-001 Detection Mechanism

The cheat scanner detects `CHEAT-001` at the **AST (Abstract Syntax Tree) level** using the `syn` crate. It traverses the AST by implementing the `Visit` trait (in `SynCheatVisitor`) and specifically inspects expressions (`visit_expr`). 

It targets **Binary Expressions** (`Expr::Binary`) where the operator is either a **Bitwise XOR (`^`)** or **Subtraction (`-`)**.

When such an operation is found, it converts both the left and right operands into strings (removing all spaces via `quote::quote!(...).to_string().replace(" ", "")`) and checks for three specific patterns:

1. **Direct Cancellation (`A ^ A` or `A - A`)**
   If the stringified left operand exactly matches the stringified right operand, it flags a violation.

2. **Wrapped Cancellation on the Left (`A.wrapping_add(B) ^ A` / `A.wrapping_sub(B) - A`)**
   It checks if the left operand is a method call to `wrapping_add` or `wrapping_sub`. If it is, it stringifies the receiver (`A`) of that method call. If the stringified receiver exactly matches the stringified right operand, it flags a violation.
   *Note: This is only checked if the receiver `A` qualifies as a "simple expression".*

3. **Wrapped Cancellation on the Right (`A ^ A.wrapping_add(B)` / `A - A.wrapping_sub(B)`)**
   It checks if the right operand is a method call to `wrapping_add` or `wrapping_sub`. If it is, it stringifies the receiver (`A`) of that method call. If the stringified receiver exactly matches the stringified left operand, it flags a violation.
   *Note: This is also only checked if the receiver `A` qualifies as a "simple expression".*

#### "Simple Expressions"
To prevent false positives or overly complex string matching, the wrapped cancellation checks require the receiver (`A`) to be a "simple expression", defined recursively by the `is_simple_expr` helper function as one of the following:
- **Path** (e.g., `variable`)
- **Field access** (e.g., `struct.field`, where the base is also a simple expression)
- **Index access** (e.g., `array[i]`, where both the array and index are simple expressions)
- **Literal** (e.g., `1`)
- **Reference** (e.g., `&variable`, where the referenced value is a simple expression)
