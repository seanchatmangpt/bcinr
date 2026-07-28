# PDDL oracles

Not yet wired. This file states what would be checked and by what, so the gap
is visible rather than absent.

## VAL (KCL-Planning/VAL, BSD-3-Clause, C++)

VAL is the standard plan validator: given a domain, a problem and a plan, it
decides whether the plan is valid. That is exactly the judgement our planners
make and currently nothing independent checks — every `Found` this repository
produces is checked only by the code that produced it.

The three areas where an independent validator would bite hardest are the ones
changed most recently, each of which was verified by a single hand-built probe:

- **`at end` conditions** — until recently every durative condition was
  evaluated once at scheduling time, so a condition true at start and false at
  completion still yielded `Found`. VAL judges conditions at the instants PDDL
  says they hold.
- **Trajectory constraints** — the legacy rail silently dropped every form but
  `always`; it now refuses them. VAL checks all of them, so it can distinguish
  "we correctly refuse" from "we refuse something valid".
- **Durative-action semantics generally** — invariants over `over all`, effect
  timing, and the no-self-overlap rule.

The harness should emit `Found` plans in VAL's plan format, run VAL, and treat
`Plan invalid` on a plan we returned as a finding in our planner. A plan VAL
rejects is not a tolerance to widen.

VAL is BSD-3-Clause, which *is* compatible with this tree — but it stays under
`vendor/` with the others anyway, because an oracle that is vendored is an
oracle that can drift into being maintained here, and its independence is the
only thing that makes it worth having.

## Others worth adding

- **ENHSP** (numeric planning) — would independently check the numeric
  precondition the legacy rail is known to drop (`semantic_falsifier.rs`
  `test_numeric_cost`, ignored with that reason recorded).
- **Fast Downward** — classical, for plan existence rather than plan validity:
  a domain where we return `Exhausted` and it returns a plan is a completeness
  finding. Note our temporal rail's greedy at-start scheduling is known to be
  incomplete, so disagreements there are expected and must be classified, not
  counted.
