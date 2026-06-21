# How to Write a Counterfactual-Mutant Test to Validate Your Oracle

**Goal:** Prove that your reference oracle is actually capable of detecting bugs, by checking it rejects deliberately wrong implementations (mutants). This is what stops a test from "passing" against a tautology.

**Prerequisites:** You have a fast branchless implementation and a separate, slow-but-obviously-correct *reference* (oracle). The danger: if the reference's body is identical to the implementation, equivalence tests prove nothing — the cheat-scanner flags this as `CHEAT[CIRCULAR_REF]` (see [guide-7](./guide-7.md)). Mirror the existing mutant suites in [`mask.rs`](../../../crates/bcinr-logic/src/mask.rs).

## Steps

1. Write the reference independently of the implementation. It must be derived from the *specification*, not copy-pasted from the optimized code:

   ```rust
   // Implementation under test (branchless).
   use bcinr_logic::mask::min_u32;

   // Independent oracle: a plain, readable definition of "minimum".
   fn min_reference(a: u32, b: u32) -> u32 {
       if a <= b { a } else { b }
   }
   ```

2. Assert equivalence on representative and boundary inputs (or via `proptest` for full coverage):

   ```rust
   #[test]
   fn test_equivalence() {
       for (a, b) in [(5, 3), (3, 5), (7, 7), (0, u32::MAX), (u32::MAX, 0)] {
           assert_eq!(min_u32(a, b), min_reference(a, b));
       }
   }
   ```

3. Define mutants: small, plausible corruptions of the reference. Each should change behaviour on at least one input:

   ```rust
   fn mutant_1(a: u32, b: u32) -> u32 { min_reference(a, b).wrapping_add(1) } // off by one
   fn mutant_2(a: u32, b: u32) -> u32 { if a <= b { b } else { a } }          // returns the max
   fn mutant_3(a: u32, b: u32) -> u32 { a }                                   // ignores b
   ```

4. Write counterfactual tests asserting the oracle *disagrees* with each mutant. If the oracle were too weak (or circular), these would fail — that is the proof of detective power:

   ```rust
   #[test]
   fn rejects_mutant_1() { assert_ne!(min_reference(2, 2), mutant_1(2, 2)); }
   #[test]
   fn rejects_mutant_2() { assert_ne!(min_reference(2, 5), mutant_2(2, 5)); }
   #[test]
   fn rejects_mutant_3() { assert_ne!(min_reference(5, 2), mutant_3(5, 2)); }
   ```

## Verify it worked

- All tests pass:

  ```bash
  cargo test -p bcinr-logic --lib rejects_mutant
  ```

- The suite has teeth: temporarily make `mutant_3` return `min_reference(a, b)` and confirm `rejects_mutant_3` now *fails*. A mutant your test cannot kill is a gap in coverage. Revert afterward.
- The cheat-scanner is satisfied (no `CIRCULAR_REF`): `cargo make scan-cheats`.

See also: [Run only the library tests](./guide-5.md), [Run the cheat-scanner and contract-gate](./guide-7.md), [PhD Gates](../reference/phd_gates.md).
