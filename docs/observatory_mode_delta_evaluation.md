### 1. Clarification on `ModeDelta` Variants
The enum `ModeDelta` is defined in `crates/bcinr-cmca/src/observatory.rs`. However, it does not have a `Switch` variant. The two variants are actually `Retain` and `ProposeDelta`:

```rust
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ModeDelta {
    Retain,
    ProposeDelta,
}
```
*(Note: A mode "switch" is handled as a separate stage via `CertifiedModeSwitch` in `mode_switch.rs`.)*

### 2. Branchless Evaluation
The system evaluates these variants in a strictly branchless manner (complying with the project's $CC=1$ mandate) inside the `evaluate_calibration` function (`observatory.rs`):

```rust
let is_unadmitted = const_eq_u32(artifact.proposal as u32, ModeDelta::Retain as u32);
let is_recert = kappa_under_on & (!gamma_under_off) & (!is_unadmitted);
```

- **Avoidance of conditional logic**: It casts the enum discriminants to `u32` and performs a constant-time equality check (`const_eq_u32`). The result is `1` if it is `Retain`, and `0` otherwise.
- **Bitwise Composition**: The `is_unadmitted` flag is then combined with other numerical stability constraints (like condition number bounds `kappa_under_on` and Gram non-degeneracy `!gamma_under_off`) using bitwise AND (`&`). This yields the `is_recert` mask without using a single `if` statement or short-circuiting operator.

### 3. Selection of the Proposed Delta
Once evaluated, the control delta is chosen via a constant-time branchless select (`const_select_u32`):

```rust
let proposed_delta = SignedFixed::from_value_bits(const_select_u32(is_recert, 1, 0) as i32);
```
- If the system allows the mode change (i.e., `ModeDelta::ProposeDelta` was passed AND all telemetry masks pass), the `proposed_delta` becomes `1`.
- If `ModeDelta::Retain` was used, or any stability mask failed, `is_recert` is `0`, and the `proposed_delta` stays `0`.

### 4. Application to `ObservatoryOutcome` and State
The resulting `proposed_delta` is sealed into a `ModeProposal`, which is bundled with the telemetry flag set into the `ObservatoryOutcome`. 

Crucially, **the `ObservatoryOutcome` itself does not mutate any persistent state**, abiding by the strict "masked-commit law" (AGENTS.md §10). Instead:
- If `ModeDelta::Retain` was evaluated, the flag `ObservatoryFlag::ModeDeltaUnadmitted` is bitwise OR'd into the `ObservatoryFlagSet`. This causes `outcome.flags.telemetry_admissible()` to return false.
- The `ObservatoryOutcome` is merely a sealed proposal. If inadmissible, it is blocked downstream in `admit_proposal` (`proposal.rs`), yielding `ProposalRefusal::TelemetryStandingBlocked`.
- If admissible (i.e., `ProposeDelta` was successful), it must successfully pass through `admit_proposal`, `seal_certificate`, and finally reach `apply_mode_switch` (in `mode_switch.rs`). Only there is the transition (`mode_digest`, `generation`) applied to the persistent `ModeState`.
