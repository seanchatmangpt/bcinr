# The Radon Law ($CC=1$) and Branchless Execution in BCINR

## The Mathematical Mandate for Cyclomatic Complexity = 1
In the BCINR framework, the **Radon Law ($CC=1$)** mandates that the cyclomatic complexity of all authoritative functions must be exactly 1. This means that execution must be strictly linear, containing absolutely no control-flow branching. The core principle driving this mandate is to create a deterministic, bounded computational substrate where the execution path—what `AGENTS.md` calls the "fixed instruction shape"—is completely decoupled from the data it processes.

By eliminating all `if` statements, `match` blocks, data-dependent loops, and early returns, the substrate ensures that for any admitted input, the execution guarantees a mathematically deterministic output with exactly the same instruction trace and resource expenditure. This mathematically bounded behavior forms a "hard substrate" designed to make timing side-channels physically impossible.

## Expressing Logic as Bitwise Polynomials
Because traditional control-flow mechanisms are forbidden (Rule 8 in `AGENTS.md`), semantic branching must be mathematically transformed into constant-time arithmetic. In BCINR, this is governed by the **Mask-Based Execution Law** (Rule 9).

Logic is expressed as bitwise polynomials where predicates become full-width boolean masks ($m \in \{0, 2^w-1\}$). Sequential semantic decisions are encoded as arithmetic selections using operations equivalent to:
$$ \operatorname{select}(m, a, b) = (m \land a) \lor (\neg m \land b) $$

Instead of branching via `if/else`, the code computes both potential outcomes and uses the predicate mask to bitwise-select the correct result. This philosophy ("Bit-parallel mechanics over byte-sequential control flow") forces all state transitions to happen through fixed-width, straight-line arithmetic, thereby maintaining the $CC=1$ guarantee at the processor instruction level.

## The Prohibition of Language-Generated Panics and Hidden Trait Branches
The strictness of the Radon Law applies not just to surface-level syntax, but transitively to the **entire object-code call graph** (Rule 7). The constitution explicitly rejects the claim "The function contains no `if`, therefore it is branchless." 

Hidden branches, such as language-generated panic paths (e.g., from indexing bounds checks or `unwrap()` calls), trait method monomorphizations, or macro expansions, represent implicit control flow that the compiler injects under the hood. If a runtime bounds check fails, the resulting panic unwinding introduces a branch, violating the invariant that instruction shape must never depend on semantic input. Therefore, any source-level abstraction—whether it's an `Option/Result` branch, an iterator short-circuit, or a hidden trait branch in a dependency—that produces an input-dependent conditional jump in the final release object code is an absolute violation of the substrate's deterministic guarantees.
