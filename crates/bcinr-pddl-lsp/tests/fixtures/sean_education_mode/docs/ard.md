# Education Mode ARD

## Status: ADMITTED

## Architecture

The education-mode lifecycle is a PDDL8 domain with 5 parallel lanes:

1. **Career lane**: interviews.json → slot → confirmed → prep
2. **LinkedIn lane**: topic → draft → review → publish
3. **Newsletter lane**: restart → draft → review → publish
4. **YouTube lane**: topic → outline → script → record → publish
5. **Rust lane**: lesson → example → tests → publish

Each lane terminates at a receipted publish action.
The `emit_education_receipt` action gates on all 5 lane completions.
The `publish_education_week` action requires all 8 preconditions (Need9 boundary).

## Receipt Chain

All publish actions require `.bcinr/receipts/<platform>-<id>.json` with `goal_reached: true`.
The education-week receipt lives at `.bcinr/receipts/education-week.json`.
