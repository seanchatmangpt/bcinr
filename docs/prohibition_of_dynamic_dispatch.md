# The Absolute Prohibition of Dynamic Dispatch and Indirect Calls in BCINR

In the BCINR (BranchlessCInRust) deterministic substrate, achieving mathematically pure, $O(1)$ cycle predictability is governed by the **Radon Law ($CC=1$)** and a strict set of runtime constitutional laws. Central to this mandate is the absolute prohibition of dynamic dispatch (`dyn Trait`) and indirect calls within the authoritative hot path.

Below is an analysis of why these constructs fundamentally violate the substrate's constant-time guarantees and how they disrupt required object-code structural audits.

## 1. Why They Violate Constant-Time Guarantees

Dynamic dispatch fundamentally undermines the axiomatic requirement that **"the authoritative instruction shape must not depend on semantic input."**

* **Hardware-Level Control Flow Hazards:** Dynamic dispatch relies on virtual method tables (vtables) to resolve function calls at runtime. At the machine level, this compiles into an indirect jump instruction. Because the jump target is evaluated at runtime based on the object's concrete type, it acts as an implicit, data-dependent branch. This directly violates the $CC=1$ rule.
* **Timing Side-Channels:** Indirect jumps are highly susceptible to CPU branch prediction mechanics. Mispredicting an indirect jump target causes CPU pipeline flushes and variable execution delays. In a system built as a "hard substrate" for AGI—where timing side-channels must be physically impossible and logic executed purely as bitwise polynomials—this hardware-level indeterminism is a fatal flaw.
* **The Zero-Allocation Boundary:** While it is possible to use unboxed dynamic traits (`&dyn Trait`), polymorphic objects in Rust are overwhelmingly accompanied by heap allocations (e.g., `Box<dyn Trait>`). Banning dynamic dispatch removes a major vector for accidental heap allocations in a `#![no_std]`, zero-allocation runtime.

## 2. Impact on Object-Code Structural Audits

BCINR's constitution mandates that source-level compliance is necessary but insufficient. The `@turing_machine` (Enforcer of Determinism) must perform rigorous, target-specific object-code disassembly audits to verify that the **complete transitive call graph** is branchless.

* **Opaque Call Graphs:** Indirect calls obscure the call graph. When a function pointer or vtable is used, static analysis tools cannot definitively trace which concrete instructions will be executed. This makes it impossible for the auditor to statically identify all reachable authoritative symbols.
* **Inability to Verify Bounded Execution:** Because the exact callee is unknown at compile time, it is impossible to statically prove that the destination function complies with absolute substrate laws—such as possessing zero loop backedges, zero panic handlers, and zero conditional jumps. A single indirect call breaks the transitive chain of trust required for formal object-code verification.

## 3. Lawful Alternatives

To preserve polymorphic capabilities without sacrificing perfect execution determinism, BCINR relies entirely on compile-time mechanics:

1. **Static Monomorphization:** Replacing `dyn Trait` with generic trait constraints (`impl Trait`) forces the Rust compiler to generate distinct, specialized copies of functions for every concrete type at compile time. This ensures all calls remain direct and static.
2. **Const Generics and Unrolling:** All variable-sized structures are forced into compile-time fixed bounds via const generics, allowing LLVM to aggressively inline functions and unroll loops into flat, straight-line assembly.
3. **Mask-Based State Selection:** Where varying behaviors are needed at runtime, BCINR encodes the logic algebraically using **bit-parallel mask multiplexing**. Predicates are computed as full-width masks (e.g., `m ∈ {0, 2^w-1}`), and state transitions happen unconditionally via arithmetic selection: `select(m, candidate, current)`.
