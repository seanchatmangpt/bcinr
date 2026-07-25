# DPAG Topology: Zero-Allocation Static Memory Layout

Based on the exploration of `crates/bcinr-powl/src/admit.rs`, the **DPAG (Dynamic Process Admission Graph)** topology implements an O(1) phase admission system relying entirely on a zero-allocation, branchless design.

Here is the breakdown of its zero-allocation memory layout and architecture:

### 1. `AdmissionContext` (Packed Bitfield)
The input context is passed as a single **packed 64-bit integer (`u64`)** instead of a heap-allocated structure. This avoids heap allocations entirely and allows branchless bitwise mask extraction.
*   **Bits 0..3**: `tenant_class` (0..3) — 0=free, 1=standard, 2=enterprise, 3=sovereign
*   **Bits 4..7**: `urgency_tier` (0..15) — Higher = more urgent
*   **Bits 8..11**: `resource_load` (0..15) — Higher = more saturated
*   **Bit 12**: `has_sla_token` (0/1)
*   **Bit 15**: `is_compensating` (0/1)

### 2. `AdmissionParameters` (Threshold Data)
The thresholds mapping contexts to topologies are defined in a flat, fixed-width struct of `u64` values:
```rust
pub struct AdmissionParameters {
    pub load_saturation_threshold: u64,
    pub urgency_priority_threshold: u64,
    pub tenant_class_priority_min: u64,
    pub tenant_class_standard_min: u64,
    pub sla_required: u64,
}
```

### 3. `AtomicAdmissionParameters` (Lock-Free Global State)
To enforce the `no_alloc` rule while maintaining thread safety, parameters are stored dynamically via a completely atomic wrapper. This allows the autonomic MAPE-K loop to mutate states dynamically with pure `Ordering::Acquire`/`Ordering::Release` semantics, circumventing `Mutex` heap allocations entirely.
```rust
pub struct AtomicAdmissionParameters {
    pub load_saturation_threshold: AtomicU64,
    // ...
}
pub static GLOBAL_ADMISSION_PARAMETERS: AtomicAdmissionParameters = ...
```

### 4. `ProcessTopology` (`#[repr(u8)]` Enum)
The eventual admitted routing topology is defined as a 1-byte enumerable structure aligned directly to a scalar `u8` discriminant:
```rust
#[repr(u8)]
pub enum ProcessTopology {
    Priority = 0,
    Standard = 1,
    Background = 2,
    Quarantine = 3,
}
```
In the `admit_dpag` function, purely branchless primitives (e.g. `ge_mask` and `select` bitwise multiplexing) compute a `v_final` integer between `0` and `3`. This value behaves as an index for a tiny stack-allocated constant array (`const TOPOLOGIES: [ProcessTopology; 4]`). As a result, the entire computation operates in constant time without any conditional jumps or memory allocations.
