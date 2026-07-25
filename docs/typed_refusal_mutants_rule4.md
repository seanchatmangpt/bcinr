# Typed-Refusal Mutant Requirement (`@armstrong_fault`)

According to **Rule 4** of the `bcinr` constitution (`AGENTS.md`), the `@armstrong_fault` role is the Master of Failure Law and governs adversarial test architecture, counterfactual mutants, and negative-domain testing. 

A critical rule for testing hostile mutants is the **Typed-Refusal Requirement**: verifying mutant detection via a generic inequality assertion (such as `assert_ne!`) is strictly prohibited. The test must definitively prove that the corrupted implementation either triggers a specific bounded typed refusal or violates a named postcondition.

### Why `assert_ne!` is Prohibited
Using `assert_ne!(baseline, mutant)` is classified under **CHEAT-009 ("Mutant theater")**. A generic inequality check only proves that the output changed. It fails to demonstrate that the substrate successfully detected and refused the *specific* unlawful state the mutant introduced.

### The Standard
The codebase requires that the test harnesses assert equality against an expected explicit `StabilityRefusal` typed error:

```rust
assert_eq!(
    result,
    Err(StabilityRefusal::ContractionMarginInsufficient)
);
```

Where a mutant produces an accepted value instead of a refusal, the independent oracle must explicitly identify the exact violated mathematical law, avoiding "it's just different" checks.

### Codebase Examples

**1. Explicit `StabilityRefusal` Assertions**
In `crates/bcinr-powl/src/admit.rs`, the mutant runner forces tests to explicitly detect the oracle mismatch and returns a mapped refusal:
```rust
    let result = verify_mutant_failure(admit_dpag_mutant_1);
    assert_eq!(result, Err(StabilityRefusal::ContractViolation));
```

Similarly, in `crates/bcinr-cmca/tests/case_studies.rs`, tests assert on specific domain refusals:
```rust
    let res = allocate(...).into_result();
    assert_eq!(res, Err(StabilityRefusal::CertificateDigestMismatch));
```

**2. Sealed APIs and Bit-Level Flags**
For branchless components that return packed flag structures rather than Enums, `crates/bcinr-cmca/tests/hostile_mutants.rs` demonstrates checking the exact failure bits (e.g., `NumericFaultSet`) rather than the overall result:
```rust
    assert_eq!(
        c.faults().bits(),
        bcinr_cmca::fixed::NumericFaultSet::OVERFLOW
            .union(bcinr_cmca::fixed::NumericFaultSet::SATURATION)
            .bits(),
        "Mutant 6 should falsely report OVERFLOW|SATURATION..."
    );
```

And querying explicit dropped constraints:
```rust
    assert!(
        !result.flags.contains(ObservatoryFlag::Drifting),
        "mutant M05 (zeroing d_js) should erase the Drifting flag that the true d_js would have set"
    );
```

These strict assertions guarantee that tests do not merely capture random deviations, but effectively exercise the underlying constitutional contracts of the deterministic substrate.
