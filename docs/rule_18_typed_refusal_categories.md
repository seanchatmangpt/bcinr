Here is the research on "Typed refusals" under Rule 18 from `AGENTS.md`:

### Requirement for Bounded Typed Refusal Codes
Under Rule 18, all rejected authoritative operations must produce a **bounded typed refusal code**. The rule explicitly dictates that **human-readable text belongs outside the hot path**. 

Furthermore, when an operation is rejected, no unsupported input may panic, silently clamp outside the admitted policy, drop a factor, fall back to a simpler algorithm, mutate partial state, or return a plausible default.

### Examples of Required Categories
Here are 6 examples of the required typed refusal categories listed in the rule:
1. `ContractViolation`
2. `UnsupportedDomain`
3. `NumericRangeExceeded`
4. `DigestMismatch`
5. `CertificateMissing`
6. `CheatDetected`
