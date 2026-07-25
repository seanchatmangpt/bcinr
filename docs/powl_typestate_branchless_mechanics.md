# Typestates and Branchless Mechanics in POWL v2

The `bcinr-powl` crate leverages a strict typestate machine paired with branchless execution tokens to provide a deterministic, zero-allocation runtime. This design ensures absolute compliance with the project's constitutional mandate of constant-time transitions (`CC = 1`) and bounded memory execution.

## 1. Phase-Indexed Typestate Machine (`PowlRunner`)

The lifetime of a workflow run is governed by a static, compile-time sequence of phases. Because transitions consume `self` by value and require specific phase markers, they eliminate the need for runtime branch checks (e.g., no `if is_running()` checks).

The phase lattice guarantees one-way, mutually exclusive transitions:
1. **`Unvalidated`**: Initial tape configuration before any checks.
2. **`Compiled`**: Advanced via `.validate()`. The tape structure is verified for cycles, size limits, and validity without mutating state.
3. **`Scheduled<KIND>`**: Advanced via `.schedule::<KIND>()`. A scheduling topology (e.g., `Standard`, `Priority`) is embedded into the type system as a const generic parameter.
4. **`Executing<KIND>`**: Advanced via `.begin_execution()`. The runner yields a linear `ExecutionToken` to track in-flight work.
5. **`Receipted<KIND>`**: Advanced via `.complete(token)`. A terminal phase that consumes the token, validates all accumulated execution properties, and produces a verifiable `Receipt`.

Because invalid state transitions simply do not compile, the hot path avoids branching entirely. 

## 2. Branchless Linear Execution Tokens (`ExecutionToken`)

During the `Executing` phase, runtime operations are recorded using an `ExecutionToken`. It serves as a bounded linear resource that represents the remaining work on the tape.

**Linearity Emulation:**
- **No Copies:** The token does not implement `Clone` or `Copy`.
- **Exhaustion Enforcement:** In debug builds, a custom `Drop` destructor bomb panics if the token is discarded before all tasks are fired.
- **Consumption:** Terminal methods like `complete` consume the token by value, bypassing the drop bomb and enforcing safe commit semantics.

**Branchless Mechanics (`CC = 1`):**
To satisfy the rule of *zero data-dependent branches*, the token avoids branching on operational defects (like double-firing or out-of-bounds execution). Instead, defects are accumulated into bitwise status registers using boolean arithmetic:

1. **Out-of-Bounds Detection:**
   ```rust
   let invalid = op_bit & !self.valid_mask;
   self.defect_invalid |= invalid;
   ```
2. **Double-Fire Detection:**
   ```rust
   let target_valid = op_bit & self.valid_mask;
   let present = self.remaining & target_valid;
   let double_fired = target_valid ^ present;
   self.defect_double_fire |= double_fired;
   ```
3. **Malformed Fires (zero or multi-bit bounds):**
   ```rust
   let is_zero = (op_bit == 0) as u64;
   let is_multi = ((op_bit & op_bit.wrapping_sub(1)) != 0) as u64;
   let malformed_flag = is_zero | is_multi;
   ```

Defects are only interrogated at the transactional boundary (when finalizing the transition to `Receipted`). If any defect registers are non-zero, the execution is refused before persistent mutation occurs.
