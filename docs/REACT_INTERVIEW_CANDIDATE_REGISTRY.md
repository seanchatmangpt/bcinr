# REACT Interview Candidate Registry

**Version:** 26.7.17 | **Document Type:** Reference Registry
**Last Updated:** 2026-07-25 | **Status:** ACTIVE

## Overview

The React Interview Candidate Registry maintains a canonical registry of candidates
participating in the Chicago REACT interview pipeline. Each candidate entry is immutable
once recorded and serves as the authority for demographic, experience, and assessment data
across all interview runs.

## Schema

### Candidate Object

```json
{
  "id": "candidate-{uuid}",
  "external_id": "string",
  "name": "string",
  "email": "string",
  "experience_years": integer,
  "self_reported_proficiency": "BEGINNER|INTERMEDIATE|ADVANCED|EXPERT",
  "background": {
    "education": "string",
    "previous_roles": ["string"],
    "open_source_projects": ["string"]
  },
  "registration_timestamp": "ISO8601",
  "registration_source": "LINKEDIN|REFERRAL|JOB_BOARD|INTERNAL",
  "previous_interview_runs": ["run_id"],
  "current_status": "SCREENING|ACTIVE|COMPLETED|WITHDRAWN"
}
```

### OCEL ObjectType Definition

**ObjectType Name:** `Candidate`

| Attribute | Type | Semantics | Mutability |
|---|---|---|---|
| candidate_id | string (UUID) | Unique identifier | Write-once |
| external_id | string | External ATS/HRMS identifier | Write-once |
| name | string | Full name | Write-once |
| email | string | Contact email | Write-once |
| experience_years | integer | Self-reported React experience (years) | Write-once |
| self_reported_proficiency | enum | BEGINNER, INTERMEDIATE, ADVANCED, EXPERT | Write-once |
| education | string | Degree/certification | Write-once |
| registration_timestamp | timestamp | When candidate joined pipeline | Write-once |
| registration_source | enum | LINKEDIN, REFERRAL, JOB_BOARD, INTERNAL | Write-once |
| current_status | enum | SCREENING, ACTIVE, COMPLETED, WITHDRAWN | Mutable (via new Selection event) |

## Example Registry Entries

### Candidate 1: Alice Chen

```json
{
  "id": "candidate-alice-chen",
  "external_id": "ext_alice_001",
  "name": "Alice Chen",
  "email": "alice.chen@example.com",
  "experience_years": 5,
  "self_reported_proficiency": "ADVANCED",
  "background": {
    "education": "BS Computer Science (UC Berkeley)",
    "previous_roles": [
      "Frontend Engineer @ StartupXYZ",
      "React Specialist @ TechCorp"
    ],
    "open_source_projects": [
      "react-query contrib",
      "custom-hooks-library"
    ]
  },
  "registration_timestamp": "2026-07-10T14:22:00Z",
  "registration_source": "LINKEDIN",
  "previous_interview_runs": [],
  "current_status": "ACTIVE"
}
```

### Candidate 2: Bob Martinez

```json
{
  "id": "candidate-bob-martinez",
  "external_id": "ext_bob_002",
  "name": "Bob Martinez",
  "email": "bob.m@example.com",
  "experience_years": 2,
  "self_reported_proficiency": "INTERMEDIATE",
  "background": {
    "education": "Bootcamp Graduate (CodePath)",
    "previous_roles": [
      "Junior Frontend Developer @ Agency1",
      "Full Stack at Startup2"
    ],
    "open_source_projects": []
  },
  "registration_timestamp": "2026-07-12T09:45:00Z",
  "registration_source": "JOB_BOARD",
  "previous_interview_runs": [],
  "current_status": "SCREENING"
}
```

### Candidate 3: Carol Singh

```json
{
  "id": "candidate-carol-singh",
  "external_id": "ext_carol_003",
  "name": "Carol Singh",
  "email": "carol.singh@example.com",
  "experience_years": 8,
  "self_reported_proficiency": "EXPERT",
  "background": {
    "education": "MS Computer Science (CMU)",
    "previous_roles": [
      "Principal Engineer @ BigTech",
      "React Architect @ MobileCorp"
    ],
    "open_source_projects": [
      "React core reviewer",
      "redux-saga maintainer"
    ]
  },
  "registration_timestamp": "2026-07-05T11:30:00Z",
  "registration_source": "REFERRAL",
  "previous_interview_runs": ["react-chicago-003"],
  "current_status": "ACTIVE"
}
```

## Interview Run Linkage

Each InterviewRun in the OCEL references exactly one Candidate via a `involves` relationship:

```json
{
  "eventType": "InterviewStarted",
  "id": "ev-interview-started-001",
  "relationships": [
    {
      "objectId": "candidate-alice-chen",
      "qualifier": "involves"
    },
    {
      "objectId": "run-react-chicago-001",
      "qualifier": "belongs_to"
    }
  ]
}
```

## Status Lifecycle

```
SCREENING → ACTIVE → COMPLETED
    ↓
  WITHDRAWN (at any point)
```

| Status | Meaning | Transitions | Triggers |
|---|---|---|---|
| SCREENING | Awaiting assignment to interview round | ACTIVE, WITHDRAWN | Admin assignment, candidate withdrawal |
| ACTIVE | Actively interviewing | COMPLETED, WITHDRAWN | Completion of all rounds or withdrawal |
| COMPLETED | Finished all interview rounds | (terminal) | Final decision (hire/reject) |
| WITHDRAWN | Candidate withdrew or ineligible | (terminal) | Candidate action or admin decision |

## Immutability & Corrections

**Registry entries are write-once.** If a candidate's information requires correction:

1. **Do not mutate** the original Candidate object
2. **Create a new InterviewObservation** documenting the correction
3. **Create a new Selection** with updated candidate_status if needed
4. **Record via PolicyStateCheckpoint** showing the corrected state

Example: Candidate Alice Chen's experience is actually 6 years, not 5.

```json
{
  "eventType": "ObservationRecorded",
  "id": "ev-correction-alice-exp",
  "timestamp": "2026-07-20T10:00:00Z",
  "attributes": [
    {"name": "finding_text", "value": "Candidate experience corrected: 5 → 6 years verified via resume review"},
    {"name": "severity", "value": "INFO"}
  ],
  "relationships": [
    {"objectId": "candidate-alice-chen", "qualifier": "about_candidate"}
  ]
}
```

Then in SelectionMade, reference the corrected finding and declare the new candidate state.

## Querying & Compliance

### Eligibility Query
**All candidates with ≥2 years React experience and INTERMEDIATE+ proficiency:**

```prolog
eligible_for_technical_round(C) :-
    candidate(C),
    candidate_experience_years(C, Y),
    Y >= 2,
    candidate_proficiency(C, P),
    member(P, [INTERMEDIATE, ADVANCED, EXPERT]).
```

### Audit Trail
**All candidates involved in completed runs in July 2026:**

```sql
SELECT DISTINCT c.name, c.candidate_id, ir.run_id, ir.round_number
FROM candidate c
JOIN interview_run ir ON ir.candidate_id = c.candidate_id
WHERE EXTRACT(MONTH FROM ir.start_time) = 7
  AND EXTRACT(YEAR FROM ir.start_time) = 2026
  AND ir.final_status IN ('PASSED_ROUND_1', 'PASSED_ROUND_2', 'REJECTED')
ORDER BY ir.start_time DESC;
```

## Data Privacy & Retention

- **PII Scope**: Names, emails, external IDs are considered PII
- **Retention**: Candidate records retained for 2 years post-completion
- **Anonymization**: For audit reports, candidate names replaced with candidate_id
- **Export**: Never export PII in OCEL logs outside secure pipeline

## Version History

| Version | Date | Changes |
|---|---|---|
| 26.7.17 | 2026-07-25 | Initial registry spec with 3 example candidates |
| 26.7.16 | 2026-07-20 | Draft schema definition |

## See Also

- `docs/REACT_INTERVIEW_CHICAGO_JTBD.md` — Interview process flow and event definitions
- `crates/bcinr-powl/src/ocel.rs` — OCEL serialization and validation
- `target/react_interview_chicago.ocel.json` — Example OCEL log with candidate data
