# The "Full-Width Mask" Mandate in BCINR (Rule 9)

In the BCINR determinism framework, all runtime predicates must resolve to full-width masks (`0` or `2^w - 1`) rather than simple `1` or `0` booleans. This specific formatting is a mathematical requirement to enable branchless execution and the canonical state selection formula.

### The Canonical Formula
The branchless replacement for an `if-else` statement is the bitwise selection formula:
```rust
select(m, a, b) = (m & a) | (~m & b)
```

### Why a Boolean (1 or 0) Fails
If `m` were merely a `1` (true) or `0` (false), applying `(m & a)` would isolate only the least significant bit of `a` and destroy the remaining `w-1` bits. 

For instance, if `a = 0xFFFFFFFF` and `m = 1`:
- `m & a` = `1 & 0xFFFFFFFF` = `0x00000001` (Data is corrupted)

Furthermore, the bitwise NOT of `1` (`~1`) typically evaluates to `0xFFFFFFFE` in a two's complement system, which would similarly mangle the bitwise evaluation of `b`.

### Why a Full-Width Mask Works
A full-width mask duplicates the boolean state across all bits of the word size `w`. 
- **True** becomes `2^w - 1` (e.g., `0xFFFFFFFF` for 32-bit), which is all `1`s.
- **False** becomes `0`, which is all `0`s.

This allows the formula to act as a pure multiplexer (MUX) at the bitwise level:

**When `m` is True (all `1`s):**
- `m & a` = `0xFFFFFFFF & a` = `a` (Preserves `a` entirely)
- `~m & b` = `0x00000000 & b` = `0` (Zeros out `b`)
- **Result**: `a | 0 = a`

**When `m` is False (all `0`s):**
- `m & a` = `0x00000000 & a` = `0` (Zeros out `a`)
- `~m & b` = `0xFFFFFFFF & b` = `b` (Preserves `b` entirely)
- **Result**: `0 | b = b`

### Conclusion
By strictly mandating full-width masks, BCINR ensures that the entirety of data structs and registers can be selected and atomically committed without ever introducing an `if` block, data-dependent jump, or loop. This fulfills the immutable `CC=1` (Radon Law) requirement and guarantees hardware-level deterministic behavior.
