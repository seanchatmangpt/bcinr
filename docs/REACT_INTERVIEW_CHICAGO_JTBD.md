# REACT Interview Domain — Chicago JTBD

**Version:** 26.7.17 | **Scope:** React Component Framework Interview Pipeline
**Status:** ACTIVE | **Audience:** Interview coordinators, process auditors, POWL pipeline engineers

## Executive Summary

The React Interview Chicago JTBD (Jobs To Be Done) defines the object model and process flow for
structured technical interviews assessing React component framework competency. This document
specifies the OCEL (Object-Centric Event Log) extension for interview domain, enabling deterministic,
auditable, and reproducible interview decision pipelines.

**Core JTBD:** Given a candidate with reported React experience, reliably assess whether they can:
1. Understand component lifecycle and state management
2. Write testable, composable component hierarchies
3. Diagnose performance bottlenecks in real-time rendering
4. Apply architectural patterns (compound components, hooks composition, context usage)

## Object-Centric Event Log (OCEL) Domain Model

### ObjectTypes

| ObjectType | Purpose | Attributes | Constraints |
|---|---|---|---|
| **InterviewRun** | A single interview session with a candidate | run_id, candidate_id, round_number, start_time, end_time | One per candidate per assessment round |
| **InterviewRound** | A distinct phase within an interview (screening, technical, architecture) | round_id, run_id, round_type, duration_minutes | Ordered by round_number |
| **Candidate** | The person being interviewed | candidate_id, name, experience_years, self_reported_proficiency | Immutable; created once |
| **Question** | A specific assessment question or task prompt | question_id, round_id, category, prompt_text, max_points | Canonicalized by round type |
| **InterviewObservation** | A discrete measurement or finding during interview | observation_id, run_id, question_id, timestamp, observer_id, finding_text | Atomic unit of evidence |
| **Selection** | A decision made based on observations | selection_id, run_id, observation_ids (many), selected_outcome, confidence, rationale | Derived from ≥1 observations |
| **PolicyState** | The state of the evaluation policy at a checkpoint | state_id, run_id, checkpoint_name, policy_version, is_passing, dominant_factors | Immutable snapshot |
| **ExecutionAction** | A policy-driven action taken post-decision | action_id, run_id, selection_id, action_type, status, timestamp | Traces policy application |
| **Receipt** | Cryptographic binding of events to a run | receipt_id, run_id, blake3_hash, event_ids, checkpoint | BLAKE3 chaining per bcinr-powl-receipt |

### EventTypes

| EventType | Payload | Relationships | Constraints |
|---|---|---|---|
| **InterviewStarted** | run_id, candidate_id, round_number | belongs_to: InterviewRun; involves: Candidate; enters: InterviewRound | One per run |
| **QuestionAsked** | question_id, round_id, timestamp | posed_in: InterviewRound; instantiates: Question | Ordered by timestamp |
| **ObservationRecorded** | observation_id, question_id, finding_text, severity | documents: Question; produced_by: Observation | ≥0 per question |
| **SelectionMade** | selection_id, observation_ids[], selected_outcome, confidence | consolidates: Observation[]; results_in: Selection; applied_to: PolicyState | 1+ per run |
| **PolicyStateCheckpoint** | state_id, checkpoint_name, is_passing, factors | captures: PolicyState; basis_events: Event[] | Immutable record |
| **ExecutionActionTriggered** | action_id, action_type, status, reason | enacts: ExecutionAction; follows: Selection; implements: PolicyState | Causally linked to selection |
| **ReceiptGenerated** | receipt_id, blake3_hash, event_ids[] | seals: InterviewRun; binds: Event[]; validates_checkpoint: PolicyStateCheckpoint | Terminal per run |
| **InterviewSealed** | run_id, final_status, total_duration | concludes: InterviewRun; final_state: PolicyState | One per run; must follow ReceiptGenerated |

## Event Flow & State Transitions

```
InterviewStarted
    ↓
QuestionAsked (0..N)
    ↓
ObservationRecorded (0..M per question)
    ↓
SelectionMade (1+ consolidations of observations)
    ↓
PolicyStateCheckpoint (immutable record of state)
    ↓
ExecutionActionTriggered (policy application)
    ↓
ReceiptGenerated (BLAKE3 seal of all events)
    ↓
InterviewSealed (termination marker)
```

### Invariants

1. **No Dangling References**: Every observation_id in a Selection must correspond to a recorded ObservationRecorded event.
2. **Temporal Ordering**: timestamp(ObservationRecorded) < timestamp(SelectionMade) < timestamp(PolicyStateCheckpoint).
3. **Causality Chain**: ExecutionActionTriggered.selection_id must reference a prior SelectionMade event.
4. **Receipt Completeness**: ReceiptGenerated.event_ids must include all events from InterviewStarted to the checkpoint prior to receipt generation.
5. **Single Final State**: Only one InterviewSealed event per InterviewRun; must reference final PolicyState.
6. **No Retroactive Edits**: All object attributes are write-once; corrections require new Selection/Checkpoint events.

## Example Trace: Candidate XYZ in Round 1

**Run ID:** react-chicago-001 | **Candidate:** Alice Chen | **Round:** Technical (1)

### Events (Chronological)

1. **InterviewStarted** (t=0)
   - run_id: react-chicago-001
   - candidate_id: candidate-alice-chen
   - round_number: 1
   - Relationships: belongs_to → InterviewRun(react-chicago-001), involves → Candidate(alice-chen), enters → InterviewRound(technical-r1)

2. **QuestionAsked** (t=1)
   - question_id: q-hooks-composition
   - round_id: technical-r1
   - prompt: "Build a custom hook that composes useState and useCallback for a form handler."
   - Relationships: posed_in → InterviewRound(technical-r1), instantiates → Question(q-hooks-composition)

3. **ObservationRecorded** (t=45)
   - observation_id: obs-alice-hooks-1
   - question_id: q-hooks-composition
   - finding: "Candidate correctly used useCallback to prevent infinite loops; dependency array complete."
   - severity: PASS
   - Relationships: documents → Question(q-hooks-composition), produced_by → Observation(obs-alice-hooks-1)

4. **QuestionAsked** (t=46)
   - question_id: q-perf-optimization
   - prompt: "Identify the bottleneck in this React render tree; propose fix."
   - Relationships: posed_in → InterviewRound(technical-r1), instantiates → Question(q-perf-optimization)

5. **ObservationRecorded** (t=90)
   - observation_id: obs-alice-perf-1
   - question_id: q-perf-optimization
   - finding: "Candidate used React DevTools Profiler correctly; identified memo boundary; explained trade-offs."
   - severity: PASS
   - Relationships: documents → Question(q-perf-optimization), produced_by → Observation(obs-alice-perf-1)

6. **SelectionMade** (t=95)
   - selection_id: sel-alice-round1
   - observation_ids: [obs-alice-hooks-1, obs-alice-perf-1]
   - selected_outcome: ADVANCE_TO_ROUND_2
   - confidence: 0.92
   - rationale: "All technical questions passed; strong conceptual grasp and practical debugging skills."
   - Relationships: consolidates → Observation[obs-alice-hooks-1, obs-alice-perf-1], results_in → Selection(sel-alice-round1), applied_to → PolicyState(ps-alice-checkpoint-1)

7. **PolicyStateCheckpoint** (t=96)
   - state_id: ps-alice-checkpoint-1
   - checkpoint_name: ROUND_1_TECHNICAL_COMPLETE
   - is_passing: true
   - factors: ["hooks_competency=PASS", "perf_diagnosis=PASS", "communication_quality=PASS"]
   - Relationships: captures → PolicyState(ps-alice-checkpoint-1), basis_events → [InterviewStarted, QuestionAsked(2), ObservationRecorded(2), SelectionMade]

8. **ExecutionActionTriggered** (t=97)
   - action_id: act-alice-advance
   - action_type: SEND_ROUND_2_INVITATION
   - status: SUCCESS
   - reason: "Selection outcome=ADVANCE_TO_ROUND_2; policy applied automatically."
   - Relationships: enacts → ExecutionAction(act-alice-advance), follows → Selection(sel-alice-round1), implements → PolicyState(ps-alice-checkpoint-1)

9. **ReceiptGenerated** (t=98)
   - receipt_id: receipt-react-chicago-001
   - blake3_hash: abc123def456... (BLAKE3 of all prior events)
   - event_ids: [ev-started, ev-q1, ev-obs1, ev-q2, ev-obs2, ev-sel, ev-checkpoint, ev-action]
   - Relationships: seals → InterviewRun(react-chicago-001), binds → Event[...], validates_checkpoint → PolicyStateCheckpoint(ps-alice-checkpoint-1)

10. **InterviewSealed** (t=99)
    - run_id: react-chicago-001
    - final_status: PASSED_ROUND_1
    - total_duration: 99
    - Relationships: concludes → InterviewRun(react-chicago-001), final_state → PolicyState(ps-alice-checkpoint-1)

## Policy Rules (Prolog8 Admission)

```prolog
% Eligibility: candidate must have React experience
eligible_for_interview(C) :-
    candidate(C),
    candidate_experience_years(C, Y),
    Y >= 1.

% Round 1 Advancement Policy
advance_to_round_2(R) :-
    interview_run(R),
    selection(S, R),
    observation_count_in_round(R, 1, N),
    N >= 2,  % At least 2 questions asked
    passing_observations(R, P),
    P / N >= 0.80.  % ≥80% pass rate

% Reject Policy
reject_candidate(R) :-
    interview_run(R),
    selection(S, R),
    selection_outcome(S, REJECT).
```

## Audit Trail & Compliance

All events are immutable and timestamped. A receipt (BLAKE3 hash-chain) seals each checkpoint, enabling:
- **Reproducibility**: Replay any run with identical event sequence → identical policy state
- **Non-repudiation**: BLAKE3 signature proves no events were retroactively modified
- **Compliance**: Every selection decision is traceable to observations; every action is traceable to a selection

See `docs/REACT_INTERVIEW_CANDIDATE_REGISTRY.md` for candidate registry schema.
See `crates/bcinr-powl/src/ocel.rs` for OCEL serialization implementation.

## References

- OCEL 2.0 Specification: https://ocel-standard.org/
- bcinr-powl Receipt Verification: `crates/bcinr-powl-receipt/`
- Process Mining Conformance: `crates/bcinr-powl/src/ocel.rs::validate_against_tape`
