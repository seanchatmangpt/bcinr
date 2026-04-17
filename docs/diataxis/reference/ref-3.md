# Reference: Specification for Primitive 3$

## Signature
```rust
pub fn primitive_3(args: ...) -> ...
```

## Functional Guarantee
- **Complexity:** (1)$
- **Determinism:** Strictly time-invariant.
- **Safety Bound:** Clamped to structural maximums (if saturation-enabled).

This primitive is used primarily in the consensus synchronization plane to map state variables across massive agent topologies.
