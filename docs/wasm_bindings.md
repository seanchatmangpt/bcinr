# WebAssembly (WASM) C-Interface Boundary

In `bcinr`, the execution of the deterministic `#![no_std]` core engines (Petri Nets, YAWL, and POWL) in browser environments relies on a WASM-compatible C-interface boundary. This boundary bridges the gap between dynamic host environments (JavaScript/browser) and the strict, zero-allocation, branchless Rust core engines by using explicitly structured Foreign Function Interface (FFI) bindings.

## 1. The C-ABI Boundary Pattern
The wrapper boundary employs standard C Application Binary Interface (ABI) conventions to ensure compatibility with WASM compilation (`wasm32-unknown-unknown`). All boundary functions are exported with `#[no_mangle] pub unsafe extern "C"`. This guarantees that the browser's WASM runtime can directly invoke these bindings using recognizable primitive arguments and direct memory manipulation without going through complex Rust-specific runtime bindings.

## 2. Opaque Pointers and State Lifecycle Control
Because the `bcinr` hot path rigorously bans heap allocations (the Zero-Allocation Boundary), the lifecycle of graph structures is carefully segregated from execution.
- **Initialization (Slow-Rail):** Engine state is instantiated via factory functions like `ref_petri_create()`, `ref_yawl_create()`, or `ref_powl_program_create()`. These functions allocate the underlying Rust structures on the heap and surrender them to the WASM host as raw, opaque pointers using `Box::into_raw()`.
- **Destruction:** The host is responsible for manually freeing these objects (e.g., `ref_petri_free()`) which safely reconstructs the `Box` and drops the allocation.

This ensures that the structural memory required by the graphs is fully settled before any execution begins.

## 3. Primitive, Mask-Based Execution Signatures
To adhere to the Radon Law ($CC=1$) and bitwise polynomial execution, the FFI wrappers strip all complex data structures from the execution calls. Instead, they accept integer masks and bounded parameters:

```rust
pub unsafe extern "C" fn ref_yawl_execute_task(
    engine: *mut BYawlEngine,
    id: u16,
    join_type: u8,
    split_type: u8,
    consume_mask: u64,
    produce_mask: u64,
    cancellation_mask: u64,
    /* ... additional masks ... */
) -> bool
```

Here, structural and behavioral state is pushed through the boundary via `u64` bitmasks and `u8` enums. The C-wrapper translates these into fixed-size structures (e.g., `BYawlTask`, `JoinType`) ensuring that logic remains branchless and bounded within the core engine.

## 4. Pre-allocated Pointer Outputs (Zero-Copy)
To avoid any intermediate heap allocations when returning complex states back to the WASM host, the interface utilizes output pointers (`*mut`).

For example, when replaying a trace on a Petri Net or executing a POWL program:
```rust
pub unsafe extern "C" fn ref_powl_execute(
    prog: *mut Powl64Program,
    watchdog_deadline: u64,
    out_concur_claims: *mut u32,
    out_scopes_entered: *mut u32,
) -> bool
```
The caller provides valid memory addresses for outputs. If the engine executes successfully, it directly modifies `out_concur_claims` or `out_scopes_entered` through the pointer reference, maintaining strict $O(1)$ memory guarantees on the hot path.

## 5. Supported FFI Subsystems
The WASM wrappers currently expose three primary subsystems corresponding to `bcinr`'s deterministic execution models:

*   **Petri Nets:** Lifecycle operations to iteratively build graphs (`add_place`, `add_transition`, `add_arc`, set markings), followed by immutable trace execution boundaries (`ref_petri_replay_trace`, `ref_petri_in_language`) that map sequence pointers into `NetBitmask64` validation operations.
*   **YAWL (Yet Another Workflow Language):** Exposes `ref_yawl_execute_task` to dispatch structural operations spanning multiple split/join dynamics, evaluating execution via bitwise boolean reduction.
*   **POWL (Partial Order Workflow Language):** Provides access to workflow evaluations (`ref_powl_execute`) that inherently respect deterministic execution watchdogs (`watchdog_deadline`) preventing unbounded loops inside the WASM sandbox environment.

## Summary
The WASM C-interface serves as a strict gateway. It allows the dynamic browser environment to allocate state outside of hot-path bounds, and strictly forces inputs into raw mask properties, pointer arrays, and output references. This fulfills the `bcinr` architectural laws, ensuring that all code executing within the WASM sandbox remains purely arithmetic, zero-allocating, and perfectly branchless.
