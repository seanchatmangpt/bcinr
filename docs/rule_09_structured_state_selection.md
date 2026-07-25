Based on my research of `AGENTS.md` (specifically Rule 9 and its supporting rules), here is the explanation regarding the Mask-based execution law:

### 1. What "selection must be fieldwise and fixed-width" means for structured state

When updating a `struct` or other compound data type ("structured state"), you cannot conditionally assign or copy the entire structure at once in a way that might compile down to a branch or variable-time memory copy. Instead, you must:
*   **Fieldwise**: Apply the mask selection independently to every individual primitive field within the struct. 
*   **Fixed-width**: Ensure all operations are performed on primitives with a known, constant bit-width (like `u32` or `u64`). This allows the bitwise polynomial `(m & a) | (~m & b)` to apply uniformly across the exact same number of bits every time, taking the exact same number of CPU cycles. 

This ensures that the CPU does exactly the same amount of bounded, arithmetic work regardless of whether the structured state is being updated to the candidate or remaining as the current state.

### 2. Why `if valid { candidate } else { current }` is explicitly prohibited

This construct is explicitly outlawed because it introduces a **data-dependent control-flow branch**, which violates several core tenets of the BCINR architecture:
*   **Violates the Radon Law ($CC=1$) & Rule 8**: The project strictly prohibits `if`, `else`, `match`, and other control-flow structures in authoritative code. The cyclomatic complexity must remain exactly 1.
*   **Violates Rule 3 (Absolute runtime laws)**: The authoritative call graph must have "no data-dependent branches". An `if` statement evaluates semantic data (`valid`) to determine the instruction pointer's next step.
*   **Violates Rule 4 (Enforcer of Determinism)**: The standard states, "The authoritative instruction shape must not depend on semantic input." A branch changes the instruction shape dynamically.
*   **Creates Timing Side-Channels**: If a branch is used, CPU branch prediction, pipelining, and instruction cache misses can cause the execution time to vary depending on which branch is taken. BCINR requires a "hard substrate" where "timing side-channels are physically impossible" (`GEMINI.md`).
*   **The Required Alternative**: Logic must be expressed as arithmetic. By forcing the use of a bitwise mask (`let next = State::select(mask, candidate, current);`), the CPU always evaluates the bitwise operations in straight-line, constant-time code. This preserves absolute determinism regardless of whether the state is conceptually accepted or rejected.
