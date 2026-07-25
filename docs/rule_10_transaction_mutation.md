# Rule 10: No Mutation Before Complete Admission

In the BCINR determinism mandate, **Rule 10** strictly prohibits the speculative mutation of persistent state. The system demands that all admission criteria be completely resolved, and an admission mask derived, before any state is persistently modified. If an operation is rejected, the persistent state must remain bit-for-bit unchanged without relying on panics, unwinding, or branching rollbacks.

## Why Speculative Mutation is Banned

Speculative mutation—where state is updated directly and then reversed or discarded upon an invalidation condition—is banned for several fundamental reasons:

1. **Branchless Requirement ($CC=1$):** Reversing speculative mutations typically requires `if`/`else` control flow, which violates the zero-branch stricture.
2. **Deterministic Execution Work:** Partially applying and then reversing state changes creates data-dependent variations in execution time and memory access patterns.
3. **Safety Through Bitwise Algebra:** By requiring an explicit "commit mask", the logic is forced to formally express admission constraints as a mathematical predicate, preventing hidden or partial state corruption from leaking through unhandled error paths. 

The prohibited pattern typically looks like this:
```rust
state.mass[i] = candidate;
state.weight[i] = next_weight;

if invalid {
    // Prohibited: Branching and speculative rollback
    return Err(...);
}
```

## The Required Transaction Shape

The legally mandated transaction sequence enforces a clean separation between candidate generation and state commitment. The required shape is strictly defined as:

1. **Current immutable state:** Begin with the existing persistent state $x_t$.
2. **Fixed-size candidate state:** Compute the desired future state $x_{\mathrm{candidate}}$ in isolated, fixed-size memory.
3. **Verify all predicates:** Unconditionally execute all verification and validation logic.
4. **Derive admission mask:** Collapse all predicate evaluations into a single full-width mathematical mask ($m_{\mathrm{admitted}}$), which is either all 1s (`0xFFFFFFFFFFFFFFFF`) if admitted or all 0s (`0x0000000000000000`) if rejected.
5. **Fieldwise masked commit:** Use bitwise SWAR (SIMD Within A Register) operations to blend the candidate and current states based on the mask, unconditionally writing the result back to persistent state.

The formal mathematical law for the commit phase is:
$$x_{t+1} = \operatorname{select}(m_{\mathrm{admitted}}, x_{\mathrm{candidate}}, x_t)$$

Where:
$$\operatorname{select}(m, a, b) = (m \land a) \lor (\neg m \land b)$$

## Avoiding Heap-Backed Cloning: Fixed-Size Scratch Structures

Because the authoritative runtime strictly prohibits heap allocations (zero heap allocation law), you cannot simply perform a `.clone()` backed by the heap to create your candidate state.

Instead, the framework mandates creating the candidate state by utilizing statically bounded, fixed-size memory. "Clone the state" in the BCINR context specifically means one of three things:

1. **Copy into a fixed-size stack value:** If the state is small, a simple bitwise copy onto the stack is preferred.
2. **Use a fixed-size scratch structure:** For larger state spaces, the system allocates fixed-width scratch buffers upfront (often backed by an internal zero-allocation `BumpArena` or statically bounded arrays). The candidate is projected into this scratch space.
3. **Compute structurally:** Calculate and hold only the specific candidate field modifications as fixed-width intermediate variables.

By using stack buffers and fixed-size scratch structs, the system can compute the full candidate state off-target, derive the admission mask, and perform the fieldwise overwrite safely:

```rust
// Compute the admission mask (e.g. 0xFFFFFFFFFFFFFFFF for valid, 0x0 for invalid)
let mask = valid_mask(...);

// Fieldwise masked commit (SWAR selection) unconditionally updates memory
// For example, iterating over fixed u64 words of the struct:
next.words[i] = (candidate.words[i] & mask) | (current.words[i] & !mask);
```

This ensures zero heap allocation overhead while guaranteeing that rejected operations leave the persistent state mathematically untouched in constant time.
