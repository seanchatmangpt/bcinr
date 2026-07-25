Based on Rule 15 in `AGENTS.md` ("Independent oracle law"), here is the breakdown of what constitutes an independent oracle and what is strictly prohibited:

### What Makes an Oracle Independent
An oracle is only considered independent if it is **structurally and logically distinct** from the implementation. It is not enough for the oracle to simply live in a separate file (e.g., `tests/reference.rs`). Furthermore, the oracle must be reviewed by `@hoare_oracle`, not the implementation owner. 

Permitted independent forms include:
- Direct mathematical formula
- Hoare specification
- Abstract state machine
- Symbolic proof
- Arbitrary-precision implementation
- SAT/SMT bit-vector model
- Exhaustive reduced-domain enumerator

### Strictly Prohibited Forms of Test Oracles
The following are explicitly forbidden as they fail to establish true independence:
- Line-by-line translation of production code
- Reuse of production normalization
- Reuse of production lookup tables
- Reuse of production fixed-point helpers
- Identical control structure with `f64`
- Importing the authoritative function and wrapping it
