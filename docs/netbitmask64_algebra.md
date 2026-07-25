# NetBitmask64 Token Algebra

The `NetBitmask64` structure in `bcinr` reimagines Petri net token flow by replacing dynamic graph traversal and heap-allocated tokens with hardware-accelerated, branchless bitwise arithmetic. By restricting the Petri net to a maximum of 64 places, the entire state and topology can be mapped directly into 64-bit unsigned integers (`u64`), enforcing constant-time execution and adhering strictly to the **Radon Law ($CC=1$)**.

## 1. Topologic Bitmapping

Instead of representing places and tokens as object instances or arrays, they are mapped to discrete bits:
- **Places:** Each place is assigned a specific bit index `i` (from 0 to 63), represented as `1u64 << i`.
- **Markings:** The current state of the net (where tokens are located) is a single `u64` bitmask. If bit `i` is set to `1`, the corresponding place contains a token.
- **Pre-sets (`in_mask`):** A `u64` bitmask representing all the required input places for a given transition.
- **Post-sets (`out_mask`):** A `u64` bitmask representing all the output places that receive tokens after a transition fires.

## 2. The Branchless Token Algebra

Traditional Petri net execution requires iterating over places and conditionally checking for tokens (`if marking.has_token(place)`). `NetBitmask64` eliminates all such loops and branches using three fundamental bitwise operations: `NOT` (`!`), `AND` (`&`), and `OR` (`|`).

When a transition `t` is triggered by an event trace, the engine evaluates it through the following deterministic, straight-line algebra:

### Step A: Identify Missing Tokens
```rust
let need = t.in_mask & !marking;
```
- **`!marking`**: Flips the current state, producing a bitmask of all *empty* places.
- **`& t.in_mask`**: Intersects the empty places with the transition's pre-set.
- **Result (`need`)**: A bitmask of all tokens that are strictly required by the transition but are currently missing from the net.

### Step B: Assess Fitness Penalty (Branchlessly)
```rust
missing += need.count_ones();
```
- Instead of looping to count missing tokens, the engine uses the hardware `popcount` instruction (`count_ones()`).
- This mathematically adds the exact number of missing tokens to the error/fitness penalty counter in a single CPU cycle.

### Step C: State Alignment (Force-Enable)
```rust
marking |= need;
```
- To allow the trace to continue executing (avoiding an `if can_fire()` divergence), the engine injects any missing required tokens into the current marking via bitwise `OR`.

### Step D: Consume Pre-set and Produce Post-set
```rust
marking = (marking & !t.in_mask) | t.out_mask;
```
- **`marking & !t.in_mask`**: The `NOT` mask of the pre-set clears out the consumed tokens from the state, strictly matching the transition's exact input requirements.
- **`| t.out_mask`**: The `OR` operation merges the produced tokens into the state, activating the transition's post-set.

## 3. The Power of Bitwise Enforcement

Because `NetBitmask64` operates strictly using this token algebra, it yields several extreme guarantees:
1. **Zero Iteration:** The engine never iterates over tokens, places, or arrays during a transition firing.
2. **Zero Allocation:** Token states require precisely 8 bytes (`u64`) of stack memory. There is no heap manipulation.
3. **Branchless Execution:** The exact same sequence of CPU instructions (`AND`, `NOT`, `OR`, `POPCOUNT`) executes whether the Petri net is in a perfectly valid state or if the trace wildly violates the topology.
4. **$O(1)$ Time Complexity:** The evaluation of pre-sets and post-sets completes in $O(1)$ time (a fixed handful of clock cycles per trace event), scaling seamlessly regardless of the net's density.
