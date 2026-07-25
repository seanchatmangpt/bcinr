# Bounded Execution and Macro-Unrolling

According to **Rule 13 (No unbounded execution)** of the BCINR Deterministic Substrate Constitution (`AGENTS.md`), any iteration that relies on runtime evaluation to dictate its boundaries or termination is strictly forbidden in the authoritative hot path.

## The Prohibition of Variable-Bound Loops and `while` Constructs

Rule 13 explicitly prohibits loop constructs that depend on dynamic input, including:
- `while value > 0`
- `for item in variable_slice`
- `loop { if done { break; } }`
- `iterator.take_while(...)`

### Why are these prohibited?
1. **Violation of the $CC=1$ Law:** Variable-bound loops inherently introduce control-flow branches (to evaluate if the loop should continue or terminate). This violates the core mandate that all authoritative functions must maintain a Cyclomatic Complexity of exactly 1.
2. **Data-Dependent Execution Work:** The constitution strictly enforces "fixed bounded execution work" and forbids "data-dependent loop termination" (Rule 3). A `while` loop or a loop over a dynamically sized slice means the CPU performs variable amounts of work depending on the semantic input, breaking the strict predictability required of the substrate.
3. **Introduction of Loop Backedges:** Unbounded constructs translate to machine code containing conditional jumps and loop backedges—instructions where the program counter jumps backward to repeat a block of code. These backedges make execution paths non-linear and vulnerable to timing variations, fundamentally conflicting with the deterministic mandate.

## How Macro-Unrolling and Straight-Line Code Guarantee No Loop Backedges

To comply with the rule that "All authoritative iteration must be compile-time fixed, generated, or macro-unrolled," the runtime replaces iteration with explicit sequential execution. 

### Mechanism of Guarantee:
1. **Eliminating the Control Structure:** Instead of writing a loop that iterates $N$ times, macro-unrolling and generated code expand the operation into $N$ distinct sequential statements during compilation or generation. For example, instead of a 4-iteration `for` loop, the code is generated as four consecutive inline operations.
2. **Forcing Straight-Line Object Code:** Because there is no loop construct in the expanded abstract syntax tree (AST) or generated source, the Rust compiler is guided to emit a linear, sequential stream of machine instructions. It computes step 1, then step 2, up to step $N$, without evaluating termination conditions or executing backward jumps.
3. **Provable Boundedness:** By fixing the number of operations statically, execution guarantees bounded memory access and execution work. There is zero risk of infinite looping or data-dependent variability.
4. **Object Code Verification:** Rule 13 emphasizes that "a fixed Rust source loop is not automatically accepted" because the compiler might still introduce branching depending on how it optimizes the loop. By generating strict straight-line code, you structurally ensure that the ultimate disassembly (the object code audit dictated by Rule 20) contains zero loop backedges and zero conditional branches, strictly maintaining $CC=1$ down to the silicon.

Ultimately, these techniques align iteration with the core philosophy of BCINR: removing variable-bound execution in favor of fixed-width, bit-parallel mechanics over byte-sequential control flow.
