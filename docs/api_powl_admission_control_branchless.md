Here is the documentation detailing how branchless admission control works in POWL, based on the contents of `crates/bcinr-powl/src/admit.rs`.

# Branchless Admission Control in POWL

The POWL admission control system maps a process's `AdmissionContext` to a `ProcessTopology` using a branchless Dynamic Directed Acyclic Graph (DPAG) evaluation. This ensures exactly zero branching ($CC=1$) and zero heap allocations during the hot path. 

Here is a breakdown of the implementation details:

### 1. Admission Context Bitfield
The inputs are packed into a single 64-bit word (`AdmissionContext`) to avoid multi-field structured overhead. The bit layout is:
* **0..3:** `tenant_class` (0=free, 1=standard, 2=enterprise, 3=sovereign)
* **4..7:** `urgency_tier` (0..15, Higher = more urgent)
* **8..11:** `resource_load` (0..15, Higher = more saturated)
* **Bit 12:** `has_sla_token` (0/1)
* **Bit 15:** `is_compensating` (0/1)

### 2. Process Topologies
Processes are dynamically routed into four lanes, represented as enum discriminants ordered by priority:
* `Priority (0)`: Enterprise/sovereign tenants with an SLA token and sufficient urgency.
* `Standard (1)`: Normal execution lane.
* `Background (2)`: Best-effort, low-urgency lane.
* `Quarantine (3)`: Isolated lane for overloaded or untrusted contexts.

### 3. Branchless Math Primitives
Instead of `if/else` control flow, sequential semantic decisions are transformed into arithmetic bit-masks:
* `ge_mask(x, y)`: A bounded greater-than-or-equal-to check. Calculates the difference `(y - x - 1)` using 64-bit integer wrapping subtraction and extracts the sign bit (`diff >> 63`), returning `!0` (all bits set to 1) if `x >= y`, and `0` otherwise.
* `select(mask, active, fallback)`: A branchless multiplexer equivalent to a ternary operator, returning `(mask & active) | (!mask & fallback)`.

### 4. Branchless Logic Flow (`admit_dpag`)
The core routing occurs via sequentially applied masks (acting as a branchless if-else chain), evaluated against dynamic `AdmissionParameters`:
1. **Quarantine Check (`q_mask`)**: Uses `ge_mask` to check if `resource_load >= load_saturation_threshold`.
2. **Priority Check (`p_mask`)**: Intersects masks checking for `tenant_class >= min`, `urgency >= min`, and whether SLA is required and satisfied. The SLA token requirement check itself is a bitwise logic composition.
3. **Standard Check (`s_mask`)**: Uses `ge_mask` to check if `tenant_class >= min`.

### 5. Topology Resolution
Once the condition masks are derived, numeric discriminants representing topologies are multiplexed into a final output discriminant:
```rust
let v1 = select(s_mask, topo_s, topo_bg);
let v2 = select(p_mask, topo_p, v1);
let v_final = select(q_mask, topo_q, v2);
```
To avoid branch-bearing `match` statements when constructing the `ProcessTopology` enum from `v_final`, the system looks up the output directly in a constant stack array using `TOPOLOGIES[(v_final & 3) as usize]`.

This design rigorously adheres to the absolute rule of the BCINR repository: fixed instruction shape independent of the underlying semantic data, resulting in completely deterministic execution overhead.
