# SWAR Text Processing and String Matching Algorithms in BCINR

## Overview
SWAR (SIMD Within A Register) is a core technique used in the `bcinr` codebase to process multiple bytes simultaneously using standard 64-bit integer registers, completely avoiding hardware-specific SIMD instructions while retaining 8x data parallelism. 

In adherence to the **BCINR Deterministic Substrate Constitution** (specifically the Radon Law, where $CC=1$), these algorithms strictly avoid control flow branches (no `if`, `match`, or data-dependent `loop`), heap allocations, or dynamic dispatch. Operations that logically require conditions are transformed into bitwise polynomials, masks, and fixed-width arithmetic.

## Key Algorithms

### 1. Aho-Corasick SIMD Step (`aho_corasick_simd_step.rs`)
A fundamental transition step for parallel string matching state machines. It operates on 8 bytes simultaneously in constant time.
* **Mechanism**: It broadcasts the low byte of an auxiliary operand to all 8 lanes and XORs it against the value. This creates a per-byte match-difference.
* **Carry-Isolated Addition**: It then adds 1 to every lane independently. To prevent an overflow in one byte from cascading to the next, it uses the SWAR carry-isolated addition identity: `((a & M) + (b & M)) ^ ((a ^ b) & ~M)` where `M = 0x7F7F7F7F7F7F7F7F`.

### 2. Memchr / Target Byte Search (`simd_memchr_u8x16.rs`, `swar_str::find_byte_in_word`)
Locates the presence and position of a specific byte within an 8-byte word branchlessly.
* **Mechanism**: To find a byte `c` in a word `v`, the algorithm first XORs the word with a broadcasted `c` to turn target bytes into zero bytes.
* **Zero-Byte Detection**: It then applies the classic Hacker's Delight zero-byte detection: `(x.wrapping_sub(0x0101...) & ~x & 0x8080...)`.
* **Result**: Returns a mask with `0x80` set in every byte lane where the target byte originally appeared, taking ~5 arithmetic operations without a single branch.

### 3. ASCII Case Conversion (`ascii_to_lowercase_simd.rs`, `swar_str::to_lower_ascii_word`)
Converts 8 characters to uppercase or lowercase simultaneously, ignoring non-alphabetic bytes.
* **Mechanism**: Uses a SWAR range check to detect if bytes fall within the `A-Z` (or `a-z`) boundaries.
* **Transformation**: The range check sets `0x80` in lanes that match. This bit is then shifted right by 2 to align with bit 5 (`0x20`). A bitwise `OR` (for lowercase) or bitwise `AND NOT` (for uppercase) applies the transformation selectively without `if` statements.

### 4. Decimal Digit Parsing (`swar_str::parse_8_decimal_digits`)
Parses exactly 8 ASCII decimal digits from a `u64` word into a `u32` value, validating them in parallel.
* **Mechanism**: Subtracts `'0'` (`0x30`) from every byte lane. 
* **Validation**: Validates that all bytes were between `0-9` using SWAR overflow checks: `digits | digits.wrapping_add(0x7676...)`.
* **Combination**: Pairs adjacent digits together by multiplying the high byte by 10 and adding the low byte, doing this progressively from 16-bit to 32-bit to the final `u32` value.

### 5. Byte Classification (`swar_str::swar_classify_bytes`)
Classifies 8 bytes simultaneously into a bitfield denoting character classes.
* **Mechanism**: Performs multiple parallel range checks (e.g., `0-9`, `A-Z`, `a-z`, whitespace).
* **Packing**: Shifts the resulting `0x80` mask flags into distinct bit positions (e.g., bit 0 for digit, bit 1 for alpha, bit 2 for whitespace) and ORs them together, yielding an 8-bit classification mask for each byte lane in a single pass.

## Architectural Context
Every primitive in the `bcinr-logic` crate is treated as an executable specification. Under the **BCINR Constitution**:
- Each algorithm possesses an independent `Hoare logic` axiomatic proof (e.g., `Precondition: { val ∈ u64 }, Postcondition: { ... }`).
- Negative Mutants (e.g., `Identity bluff`, `Bit-skip bluff`) are strictly enforced in the test suite to ensure the test coverage explicitly rejects flawed logic structures.
- Because these algorithms process 8 bytes sequentially via logic gates rather than control flow, they serve as auto-vectorization hints for modern compilers, generating highly optimized hardware SIMD (SSE/AVX/NEON) where supported, while acting as an ultra-fast portable fallback on `no_std` environments.
