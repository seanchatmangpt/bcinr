# OcelCausalFrame Binary Layout

The `OcelCausalFrame` is a 128-byte, cache-aligned structure used in BCINR before performing BLAKE3 hashing. By specifying `#[repr(C, align(64))]`, it aligns exactly to cache line boundaries and occupies exactly two 64-byte cache lines.

## Structure Definition

```rust
#[derive(Clone)]
#[repr(C, align(64))]
pub struct OcelCausalFrame {
    /// Monotonically increasing step identity within a run.
    pub instruction_id: u64,
    /// Scatter of active denial lanes.
    pub fired_mask: u64,
    /// Denial polarity at the time this step was manufactured.
    pub denial: DenialPolarity, // (u64 wrapper)
    /// Up to 8 packed object references participating in this step.
    pub obj_refs: [PackedObjRef; 8], // (u32 wrappers)
    /// Wall-clock timestamp in nanoseconds.
    pub ts_ns: u64,
    /// Index into the activity table for this step's activity.
    pub activity_idx: u16,
    /// Classifier byte for the POWL node kind (XOR, SEQ, LOOP, etc.).
    pub node_kind: u8,
    /// Internal padding.
    pub pad: [u8; 5],
    /// BLAKE3 hash of the preceding frame (or genesis zeros for the first frame).
    pub prior_hash: [u8; 32],
}
```

## Memory Layout (128 Bytes, Little-Endian)

| Offset | Size (Bytes) | Field Name | Type | Description |
|:---:|:---:|:---|:---|:---|
| `0x00` (0) | 8 | `instruction_id` | `u64` | Monotonically increasing step identity |
| `0x08` (8) | 8 | `fired_mask` | `u64` | Scatter of active denial lanes |
| `0x10` (16) | 8 | `denial` | `u64` | Denial polarity (`DenialPolarity` wrapper) |
| `0x18` (24) | 32 | `obj_refs` | `[u32; 8]` | Up to 8 packed object references (`PackedObjRef` wrappers) |
| `0x38` (56) | 8 | `ts_ns` | `u64` | Wall-clock timestamp in nanoseconds |
| `0x40` (64) | 2 | `activity_idx` | `u16` | Index for this step's activity |
| `0x42` (66) | 1 | `node_kind` | `u8` | Classifier byte for the POWL node kind |
| `0x43` (67) | 5 | `pad` | `[u8; 5]` | Explicit padding for internal alignment |
| `0x48` (72) | 32 | `prior_hash` | `[u8; 32]` | BLAKE3 hash of the preceding frame |
| `0x68` (104) | 24 | *(Implicit Padding)* | N/A | Compiler-inserted padding to meet `align(64)` constraint, padding to exactly 128 bytes |

## Serialization for Hashing

When computing the rolling BLAKE3 receipt, the 128-byte memory layout is compacted into a **99-byte** buffer to remove all non-deterministic or irrelevant padding. 

The serialization buffer takes the little-endian bytes in this exact layout (`to_hash_bytes`):
1. `instruction_id` (8 bytes)
2. `fired_mask` (8 bytes)
3. `denial` (8 bytes)
4. `obj_refs` (32 bytes)
5. `ts_ns` (8 bytes)
6. `activity_idx` (2 bytes)
7. `node_kind` (1 byte)
8. *(Skipped explicit 5-byte padding)*
9. `prior_hash` (32 bytes)
10. *(Skipped implicit 24-byte padding)*
