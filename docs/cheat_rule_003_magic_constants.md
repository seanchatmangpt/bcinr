# CHEAT-003: Magic Constants

## Overview

According to Rule 16 (Anti-cheat manifesto) of the BCINR Deterministic Substrate Constitution, **CHEAT-003** prohibits the use of "magic constants"—any unexplained literal values that control production behavior. 

Examples of prohibited magic constants include:
- `0xDEADBEEF`
- `0xDEAD_BEEF`
- `0xCAFEBABE`
- `0xCAFE_BABE`

## Why Unexplained Literal Constants are Prohibited

In the BCINR deterministic framework, the authoritative runtime must preserve fixed mathematical mechanics where every operation has a contractual contribution. Unexplained literal constants violate several core tenets of the constitution:

1. **Numeric-law Requirements (Rule 14):** The constitution strictly governs how values are introduced. Rule 14 states that every smoothing or clamp constant must be explicitly:
   - Named
   - Derived
   - Admitted
   - Included in the influence digest
   Unexplained inline literals skip these mandatory admission steps and undermine the mathematical contract.

2. **Axiomatic Derivation:** A foundational standard of BCINR is that "If a property cannot be stated precisely, it is not yet law." Magic constants introduce arbitrary, undefined behavior into the substrate. The `@hoare_oracle` cannot mathematically prove the validity of a system whose states are influenced by arbitrary magic numbers without an independent mathematical specification.

3. **Prevention of Speculative Behavior:** The system relies on bounded, verifiable execution work. Any constant controlling production behavior must be structurally audited and traced back to a specific requirement, preventing untested "quick fixes" or hacks from polluting the authoritative hot path.

## Why Formatting Changes Do Not Make Them Lawful

The constitution explicitly notes that "Formatting changes do not make a constant lawful." 

This rule exists to prevent structural scanners and auditors (like the `bcinr-cheat-scanner` governed by `@turing_machine`) from being bypassed via syntactic tricks (which also relates to CHEAT-006: Scanner evasion). 

For example, changing the formatting from `0xDEADBEEF` to `0xDEAD_BEEF`, using alternate numeric bases, or splitting literals does not change the underlying semantic role of the value. The constant remains arbitrary and unadmitted. In BCINR, structural compliance is judged not just by passing a source code regex, but by proving that the logic has been properly derived and verified at the object-code level. A constant only becomes lawful when it transitions from an unexplained literal into a named, mathematically derived, and explicitly admitted contract.
