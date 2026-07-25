# Admission Control in POWL (`bcinr-powl`)

The admission control for POWL is implemented in `crates/bcinr-powl/src/admit.rs`. It achieves branchless, constant-time `O(1)` process routing using Dynamic Policy Allocation Graphs (DPAG) and bitwise arithmetic to strictly adhere to the Radon Law ($CC=1$) and zero-allocation requirements.

Here is a breakdown of how it evaluates and routes workflows branchlessly without panicking.

## 1. Packed Admission Context
Instead of using branching data structures, the `AdmissionContext` represents a process as a single 64-bit integer bitfield. The bits are structured as follows:
- `0..3`: `tenant_class` (0 = free, 3 = sovereign)
- `4..7`: `urgency_tier` (0..15)
- `8..11`: `resource_load` (0..15)
- `12`: `has_sla_token` (0/1)
- `15`: `is_compensating` (0/1)

These fields are extracted using bitwise shifts and masks, avoiding any conditional unpacking logic.

## 2. Dynamic Thresholds
The `AdmissionParameters` define thresholds dynamically adjusted by the system's MAPE-K loop (e.g., `load_saturation_threshold`, `urgency_priority_threshold`). To ensure branchless logic, these are compared against the context utilizing purely arithmetic mask functions.

## 3. Branchless Primitive Mechanics
The core components preventing branches and panics are the masking and selection functions.
- **`ge_mask(x, y)`**: Implements a branchless `>=` operator. It calculates `(y - x - 1)` using wrapping subtractions to avoid overflow panics. The difference's sign bit is then shifted down `diff >> 63`, producing `!0` (all 1s) if `x >= y` and `0` otherwise.
- **`select(mask, active, fallback)`**: Simulates an `if-else` statement utilizing a sign-mask multiplexer: `(mask & active) | (!mask & fallback)`.

## 4. Constant-Time State Selection & Routing
Workflows are never abruptly "rejected" or panicked on; instead, they are routed to one of four execution lanes (Topologies): `Priority (0)`, `Standard (1)`, `Background (2)`, or `Quarantine (3)`. Unqualified or over-saturated processes simply degrade to lower priority lanes or `Quarantine`.

The function `admit_dpag` executes the routing rules in four steps:

1. **Evaluate Masks**:
   - *Quarantine Mask* (`q_mask`): Load saturation check.
   - *Priority Mask* (`p_mask`): Combines masks for tenant class, urgency, and SLA tokens using bitwise `&`.
   - *Standard Mask* (`s_mask`): Tenant class check.

2. **Sequential Multiplexing**:
   It resolves the routing lane sequentially via bitwise selections to simulate control flow:
   ```rust
   let v1 = select(s_mask, topo_s, topo_bg);
   let v2 = select(p_mask, topo_p, v1);
   let v_final = select(q_mask, topo_q, v2);
   ```
   This guarantees the precedence: Quarantine > Priority > Standard > Background.

3. **Enum Mapping without `match` or Panics**:
   Finally, `v_final` gives an integer discriminant. Converting integers to enums via `match` or `transmute` introduces hidden branches or safety concerns. The logic safely maps it using a fixed-size array lookup:
   ```rust
   const TOPOLOGIES: [ProcessTopology; 4] = [
       ProcessTopology::Priority,
       ProcessTopology::Standard,
       ProcessTopology::Background,
       ProcessTopology::Quarantine,
   ];
   
   // Bound enforced securely with bitwise AND, entirely averting bounds-check panics.
   TOPOLOGIES[(v_final & 3) as usize]
   ```
By utilizing wrapping arithmetic, bit-mask multiplexing, and bounded stack array indexing, `admit.rs` perfectly processes complex business routing logic entirely free of runtime branching or panic potentials.
