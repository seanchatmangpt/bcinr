Here is the documentation of the "Mask Calculus" based on the implementation found in `crates/bcinr-logic/src/mask.rs`:

# Mask Calculus in `bcinr`

The "Mask Calculus" provides the mathematical foundation for branchless conditional selection and arithmetic operations within the B-Calculus framework. Its core purpose is to replace conditional branches (e.g., `if` statements)—which can cause CPU pipeline stalls due to mispredictions—with pure, constant-time bitwise arithmetic, satisfying the project's strict $CC=1$ rule.

## Representation of Boolean Truth
In Mask Calculus, boolean conditions are represented using an **all-ones / all-zeros convention**:
- **True**: Represented as all bits set to `1` (e.g., `0xFFFF_FFFF` for 32-bit values or `0xFFFF_FFFF_FFFF_FFFF` for 64-bit values).
- **False**: Represented as all bits set to `0` (e.g., `0x0000_0000`).

Mask operations rely on inputs being exactly one of these two extremes; any intermediate value produces undefined behavior according to the mathematical contract.

## Core Selection Operation
In the formal B-Calculus notation, conditional selection is defined by the function $M$:
$$ M(c, a, b) = (c \ \& \ a) \ | \ (\sim c \ \& \ b) $$

Where:
- $c$ is the condition mask (either all-ones or all-zeros).
- $a$ is the value chosen if $c$ is True.
- $b$ is the value chosen if $c$ is False.
- $\&$ is bitwise AND, $|$ is bitwise OR, and $\sim$ (or `!`) is bitwise NOT.

This identity is implemented as `select_u32` and `select_u64`.

## Algebraic Composition of Masks
Because boolean states are mapped completely across the integer's bits, standard bitwise operators serve directly as algebraic logical operators on the masks. Composing masks operates exactly like composing boolean logic:

- **Logical AND ($\land$)**: `mask1 & mask2`
  Yields `0xFFFF_FFFF` if both masks are true (all-ones), otherwise `0x0000_0000`.
- **Logical OR ($\lor$)**: `mask1 | mask2`
  Yields `0xFFFF_FFFF` if at least one mask is true, otherwise `0x0000_0000`.
- **Logical NOT ($\neg$)**: `!mask`
  Inverts the mask, turning True into False (`0xFFFF_FFFF` $\to$ `0x0000_0000`) and vice versa.
- **Logical XOR ($\oplus$)**: `mask1 ^ mask2`
  Yields True if exactly one mask is True (equivalent to logical inequality).

## Generation of Masks
Masks are generated algebraically from raw data without ever invoking a conditional jump. For example:
- **Less-than (`lt_mask_u32`)**: Evaluated as `0u32.wrapping_sub((a < b) as u32)`. On x86-64, this emits a completely branchless `SETB` + `NEG` instruction sequence.
- **Zero-test (`is_zero_mask_u32`)**: Relies on the two's-complement identity that `x | -x` propagates the highest bit for non-zero numbers.
- **Equality (`eq_mask_u32`)**: XORs the two numbers to find differences, then applies the zero-test trick on the result.

By composing these generated masks using bitwise arithmetic and passing them into the primitive $M$, complex state transitions can be evaluated in constant time $O(1)$ while maintaining a deterministic, branchless instruction shape.
