Here is the documentation on how Object-Centric Event Logs (OCEL) are structured and recorded safely in `crates/bcinr-powl/src/ocel.rs`, specifically focusing on how it respects the Zero-Allocation Boundary.

### Object-Centric Event Log (OCEL) Structure

The OCEL implementation provides a deterministic mechanism to record execution traces of POWL (Partially Ordered Workflow Language) workflows.

1. **`OcelEvent`**: Represents a discrete event in the log.
   - `event_id`: A unique sequential identifier.
   - `activity`: The event type, which is either `"op_fired"` (an operation executed) or `"run_sealed"` (a workflow run finished and is sealed with a bitmask of its fired operations).
   - `timestamp`: A monotonic tick counter to preserve the order of events.
   - `run_id`: The identifier of the execution run.
   - `op_idx`: For `"op_fired"`, the index of the operation. For `"run_sealed"`, it holds the low 32 bits of the declared `op_trace` bitmask.
   - `kind_tag`: Auxiliary tag for metadata.

2. **`OcelLog`**: The log itself is constructed to hold a bounded, deterministic number of events without ever using the heap.
   - `events`: A fixed-size array `[OcelEvent; 512]`.
   - `count`: Tracks the current number of events.
   - `tick`: A monotonic counter incremented on every event to act as a timestamp.

### Maintaining the Zero-Allocation Boundary (No Heap)

The system manages to map dynamic `run_id`s, record events, and validate traces entirely within the constraints of the **deterministic substrate** (0 heap allocations and $CC=1$ branchless execution):

1. **Fixed-Capacity Arrays**:
   Because heap structures like `Vec` or `HashMap` are banned, `OcelLog` internally stores events in a static `[OcelEvent; 512]` buffer. Appending an event is an $O(1)$ operation that simply checks if `count < 512` and inserts the event, returning a typed `OcelError::Overflow` refusal if full, rather than implicitly allocating more memory.

2. **Symmetric Run-Bounded Conformance Gating (SRBCG)**:
   During conformance checking, the engine must group events by `run_id` to validate workflow constraints. To do this without a heap-allocated Hash Map, it restricts tracking to a maximum of 64 concurrent or unique runs. It allocates a fixed-size stack array: `[u64; 64]` for `run_ids`.

3. **Branchless Comparison Network**:
   To find or allocate a slot for an incoming `run_id` without using branching (`if` / `match`), the engine uses `process_event_srbcg()`. This algorithm loops exactly 64 times over the slot array and leverages a comparison network that compiles down to unrolled conditional selection instructions (e.g., `CSEL`/`CMOV`):
   ```rust
   for i in 0..64 {
       let is_match = (run_ids[i] == incoming_rid) as usize;
       match_idx = (is_match * i) + ((1 - is_match) * match_idx);
   }
   ```
   Instead of dynamically allocating new map entries, it performs pure arithmetic selection to either return an existing slot index, allocate a new slot using `current_count`, or set an overflow bitmask if the 64-run limit is exceeded. 

4. **Bitmask Accumulation over Control Flow**:
   When verifying execution bounds (like ensuring operations aren't fired multiple times or out of order), it maps operations to bits within a `u64` bitmask (`op_trace`). Checking for duplicate fires or missing predecessors is done using bitwise logic (e.g., `pred_mask & !op_trace`) rather than iterating through dynamically sized vectors, thus strictly remaining on the deterministic, allocation-free substrate.
