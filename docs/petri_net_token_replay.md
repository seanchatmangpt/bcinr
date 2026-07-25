# Petri Net Token Replay Engine: Sequence Pointers and Branchless Execution

The Petri Net token replay engine in `bcinr` provides a deterministic, branchless verification mechanism to simulate token-flow semantics over workflow structures. To comply with the strict **Radon Law** ($CC=1$, no data-dependent branches, no variable loop termination), the system maps external trace data (sequence pointers) into pure bitwise `NetBitmask64` arithmetic.

## 1. The FFI Boundary: Bridging Sequence Pointers

In dynamic host environments (like WASM or language runtimes), execution traces are typically provided as dynamically-sized arrays of strings (e.g., event logs). Passing these directly into the core engine would violate the **Zero-Allocation Boundary** and require dynamic parsing loops.

To solve this, the wrapper interface exposes immutable trace execution boundaries (e.g., `ref_petri_replay_trace`) that accept **sequence pointers**—most commonly `activities: *const *const c_char` along with a fixed `len`. 

1. **Slow-Rail Ingestion:** The runtime iteratively traverses these sequence pointers on the "slow rail" (the outer ABI boundary), dereferencing the string labels and mapping them against the graph's static dictionary.
2. **Translation to Bitmask Sequences:** The string sequence is flattened into an array of fixed transition indices (`t_idx`). 
3. **Hot-Path Hand-off:** By the time the execution crosses into the core token replay engine, the variable-length string arrays have been completely eradicated. The engine only receives a contiguous sequence of integer identifiers.

## 2. `NetBitmask64` Validation Operations

Inside the core, the entire Petri Net topology is mathematically reduced to a `NetBitmask64` structure. Nodes and places no longer exist as heap-allocated structs; they are bit positions in a `u64` word.

Each transition is defined strictly by two validation masks:
- `in_mask`: The bitset of tokens required in the pre-set to fire.
- `out_mask`: The bitset of tokens produced in the post-set after firing.

### Branchless Token Validation ($CC=1$)

When the replay engine validates a transition mapped from the sequence pointer, it avoids all `if can_fire()` conditional logic. State mutations rely on straight-line bitwise calculus.

If a trace attempts to fire a transition, the engine must account for missing tokens (fitness penalty) without diverging the control flow:

```rust
// 1. Identify missing required tokens branchlessly
let need = t.in_mask & !marking;

// 2. Accumulate conformance penalties directly (popcount is a hardware instruction)
missing += need.count_ones();

// 3. Force-enable missing tokens to allow the trace to continue
marking |= need;

// 4. Consume the pre-set and produce the post-set
marking = (marking & !t.in_mask) | t.out_mask;
```

This ensures that the sequence of CPU instructions is perfectly identical regardless of whether the trace perfectly fits the model or violates it entirely. The $CC=1$ rule is preserved.

## 3. Trace Execution Without Variable Loops

The most challenging constraint in `bcinr` trace replay is the prohibition of variable graph traversals and data-dependent loop termination. Standard Petri Net execution uses a `while` loop to resolve silent/invisible transitions (e.g., `while changed { fire_invisible(); }`). This is illegal on the authoritative hot path.

### Const-Generic Loop Unrolling

To eliminate the `for` and `while` loops over transitions, the `bcinr` scheduler employs const-generic topologies (`const_tick<const N: usize>`). 

When iterating over possible transition firings (such as evaluating which nodes are enabled), the engine uses macro-unrolled or const-bounded `for_each` sequences:

```rust
(0..TRANSITIONS).for_each(|i| {
    let (next_state, was_fired) = self.state.try_fire(self.inputs[i], self.outputs[i]);
    self.state = next_state;
    // ...
});
```

Because `TRANSITIONS` is a compile-time constant, the Rust compiler flattens this completely. On target architectures like ARM64, this sequence compiles directly into a continuous block of `(AND + SUBS + CSINV + ORRS)` instructions. There is no loop counter, no backedge, and no branch predictor pollution.

### Fixed-Point Epsilon Closures

Instead of variable-bound loops to resolve invisible (silent) transitions between sequence pointer events, the engine precomputes the reachability matrix or imposes a strict, statically bounded number of iterations based on the topological depth of the net (`epsilon_close`).

If a network requires up to 4 sequential invisible firings, the engine executes exactly 4 branchless firing passes. If the state stabilizes after 1 pass, passes 2, 3, and 4 simply perform null-mutations (`next_state = current_state` via mask selection). 

## Conclusion

By mapping dynamic FFI sequence pointers into statically flattened transition indices, and validating those indices via `NetBitmask64` bitwise calculus, the Petri Net Token Replay Engine guarantees a perfectly deterministic, $O(1)$ constant-time-per-event execution profile. The compiler guarantees structural unrolling, physically eliminating variable loops from the generated object code.
