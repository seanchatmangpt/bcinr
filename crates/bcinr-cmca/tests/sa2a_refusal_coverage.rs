//! Hostile verification for the merged-in sa2a surface (lane 7).
//!
//! The sa2a modules ship no in-crate `#[cfg(test)]` coverage; these integration
//! tests are their only exact-variant refusal assertions. Constitution §18/§19:
//! every refusal is asserted against its EXACT typed variant (`assert_eq!` on
//! the `Err` value), never `is_err()` alone. Each guarded check carries a
//! `non-vacuity:` comment naming the guarded source path and the plausible
//! mutation that would fail the assertion.

use bcinr_cmca::sa2a::interop::InteropRefusal;
use bcinr_cmca::sa2a::*;

// ---------------------------------------------------------------------------
// admission (src/sa2a/admission.rs)
// ---------------------------------------------------------------------------

// non-vacuity: src/sa2a/admission.rs:4-5 — deleting the `e.contract != CONTRACT`
// gate, or mutating CONTRACT's literal so a foreign contract string compares
// equal, makes the first assert stop failing; deleting the `authority != "none"`
// gate makes the second assert stop failing. Both assert the exact variant.
#[test]
fn admission_refuses_wrong_contract_and_non_none_authority_exactly() {
    let base = Envelope {
        contract: CONTRACT,
        subject: "s",
        effect_id: "e",
        replay_id: "r",
        authority: "none",
    };
    assert_eq!(admit(&base), Ok(()));

    let wrong_contract = Envelope {
        contract: "some-other-contract",
        ..base
    };
    assert_eq!(admit(&wrong_contract), Err(Refusal::Contract));

    let granted_authority = Envelope {
        authority: "lease-001",
        ..base
    };
    assert_eq!(admit(&granted_authority), Err(Refusal::Authority));
}

// ---------------------------------------------------------------------------
// interop (src/sa2a/interop.rs)
// ---------------------------------------------------------------------------

// non-vacuity: src/sa2a/interop.rs:4 — deleting the `authority != "none"` gate
// lets a granted-authority envelope allocate; the exact Err(InteropRefusal::Authority)
// assert fails then. The trailing Ok assert proves the budget gate is not what
// refused (the refusal is the authority gate alone).
#[test]
fn interop_refuses_envelope_that_carries_authority_exactly() {
    let budget = ResourceEnvelope {
        cpu: 10,
        memory: 10,
        io: 10,
    };
    let want = Allocation {
        cpu: 1,
        memory: 1,
        io: 1,
    };

    let powerless = Envelope {
        contract: CONTRACT,
        subject: "s",
        effect_id: "e",
        replay_id: "r",
        authority: "none",
    };
    assert_eq!(
        bcinr_cmca::sa2a::interop::allocate_via_envelope(&powerless, &budget, &want),
        Ok(want)
    );

    let granted = Envelope {
        authority: "lease-001",
        ..powerless
    };
    assert_eq!(
        bcinr_cmca::sa2a::interop::allocate_via_envelope(&granted, &budget, &want),
        Err(InteropRefusal::Authority)
    );
}

// ---------------------------------------------------------------------------
// reservation (src/sa2a/reservation.rs)
// ---------------------------------------------------------------------------

// non-vacuity: src/sa2a/reservation.rs:79/82/85 — each guarded comparison
// (subject_id, forecast_digest, authorization_binding_digest) must refuse with
// its OWN variant; a mutation deleting one comparison, or conflating two into
// one shared variant, fails the paired exact-variant assert.
#[test]
fn reservation_refuses_subject_forecast_and_binding_mismatches_exactly() {
    let proposal = ReservationProposal {
        subject_id: 7,
        horizon_start: 0,
        horizon_end: 10,
        requested: Allocation {
            cpu: 1,
            memory: 1,
            io: 1,
        },
        forecast_digest: 0xAA,
        reason_digest: 0xBB,
    };
    let capacity = ResourceEnvelope {
        cpu: 10,
        memory: 10,
        io: 10,
    };
    let admission = ReservationAdmission {
        subject_id: 7,
        forecast_digest: 0xAA,
        authorization_binding_digest: 0xCC,
    };
    assert!(admit_reservation_proposal(proposal, &capacity, admission, 0xCC).is_ok());

    let wrong_subject = ReservationAdmission {
        subject_id: 8,
        ..admission
    };
    assert_eq!(
        admit_reservation_proposal(proposal, &capacity, wrong_subject, 0xCC),
        Err(ReservationRefusal::SubjectMismatch)
    );

    let wrong_forecast = ReservationAdmission {
        forecast_digest: 0xAB,
        ..admission
    };
    assert_eq!(
        admit_reservation_proposal(proposal, &capacity, wrong_forecast, 0xCC),
        Err(ReservationRefusal::ForecastDigestMismatch)
    );

    assert_eq!(
        admit_reservation_proposal(proposal, &capacity, admission, 0xCD),
        Err(ReservationRefusal::AuthorizationBindingMismatch)
    );
}

// ---------------------------------------------------------------------------
// predictive_envelope (src/sa2a/predictive_envelope.rs)
// ---------------------------------------------------------------------------

fn predictive_candidate() -> PredictiveConsequenceEnvelope {
    PredictiveConsequenceEnvelope {
        subject_id: 11,
        horizon_start: 100,
        horizon_end: 200,
        expected: Allocation {
            cpu: 4,
            memory: 4,
            io: 4,
        },
        lower: Allocation {
            cpu: 2,
            memory: 2,
            io: 2,
        },
        upper: Allocation {
            cpu: 6,
            memory: 6,
            io: 6,
        },
        confidence_ppm: 900_000,
        observation_digest: 0xD1,
        model_digest: 0xD2,
        provenance_digest: 0xD3,
    }
}

fn predictive_admission() -> PredictiveAdmission {
    PredictiveAdmission {
        subject_id: 11,
        observation_digest: 0xD1,
        model_digest: 0xD2,
        provenance_digest: 0xD3,
        min_confidence_ppm: 500_000,
    }
}

// non-vacuity: src/sa2a/predictive_envelope.rs:172/178 — deleting the
// observation-digest comparison (or the provenance-digest comparison) makes the
// corresponding assert stop failing. :237 — swapping the ExpectedExceedsUpper
// check to compare `lower` instead of `upper` (or dropping the memory axis from
// check_bounds) makes the third assert stop failing.
#[test]
fn predictive_envelope_refuses_digest_and_upper_bound_mismatches_exactly() {
    assert!(admit_predictive_envelope(predictive_candidate(), predictive_admission()).is_ok());

    let wrong_observation = PredictiveAdmission {
        observation_digest: 0xD4,
        ..predictive_admission()
    };
    assert_eq!(
        admit_predictive_envelope(predictive_candidate(), wrong_observation),
        Err(PredictiveRefusal::ObservationDigestMismatch)
    );

    let wrong_provenance = PredictiveAdmission {
        provenance_digest: 0xD5,
        ..predictive_admission()
    };
    assert_eq!(
        admit_predictive_envelope(predictive_candidate(), wrong_provenance),
        Err(PredictiveRefusal::ProvenanceDigestMismatch)
    );

    let mut expected_over_upper = predictive_candidate();
    expected_over_upper.expected.memory = 7; // > upper.memory = 6
    assert_eq!(
        admit_predictive_envelope(expected_over_upper, predictive_admission()),
        Err(PredictiveRefusal::ExpectedExceedsUpper {
            axis: ResourceAxis::Memory
        })
    );
}

// ---------------------------------------------------------------------------
// predictive_measure (src/sa2a/predictive_measure.rs)
// ---------------------------------------------------------------------------

// non-vacuity: src/sa2a/predictive_measure.rs:64/68/72 — deleting the subject,
// forecast-digest, or standing-digest comparison makes the corresponding exact
// assert stop failing; a mutation reordering the three checks still fails
// because each input tampers exactly one field.
#[test]
fn predictive_measure_refuses_identity_mismatches_exactly() {
    let capacity = ResourceEnvelope {
        cpu: 100,
        memory: 100,
        io: 100,
    };
    let standing = ForecastStanding {
        sample_count: 4,
        mean_squared_error: 1,
        mean_absolute_error: 1,
        root_mean_squared_error: 1,
        signed_bias: 0,
        interval_coverage_ppm: 950_000,
        shock_misses: 0,
        shock_false_positives: 0,
    };

    let admitted_forecast =
        admit_predictive_envelope(predictive_candidate(), predictive_admission()).unwrap();
    let artifact = build_predictive_measure(&admitted_forecast, &capacity, standing);

    let admission = PredictiveMeasureAdmission {
        subject_id: artifact.subject_id,
        forecast_digest: artifact.forecast_digest,
        standing_digest: artifact.standing_digest,
    };
    assert!(admit_predictive_measure(artifact, admission).is_ok());

    let wrong_subject = PredictiveMeasureAdmission {
        subject_id: artifact.subject_id + 1,
        ..admission
    };
    assert_eq!(
        admit_predictive_measure(artifact, wrong_subject),
        Err(PredictiveMeasureRefusal::SubjectMismatch)
    );

    let wrong_forecast = PredictiveMeasureAdmission {
        forecast_digest: artifact.forecast_digest ^ 1,
        ..admission
    };
    assert_eq!(
        admit_predictive_measure(artifact, wrong_forecast),
        Err(PredictiveMeasureRefusal::ForecastDigestMismatch)
    );

    let wrong_standing = PredictiveMeasureAdmission {
        standing_digest: artifact.standing_digest ^ 1,
        ..admission
    };
    assert_eq!(
        admit_predictive_measure(artifact, wrong_standing),
        Err(PredictiveMeasureRefusal::StandingDigestMismatch)
    );
}

// ---------------------------------------------------------------------------
// predictive_model (src/sa2a/predictive_model.rs)
// ---------------------------------------------------------------------------

const TRACE_PROVENANCE: u64 = 0xE1;
const MODEL_DIGEST: u64 = 0xE2;
const RUNTIME_PROFILE: u64 = 0xE3;

fn trace() -> TraceWindow<1, 2> {
    TraceWindow {
        subject_id: 21,
        samples: [
            TraceSample {
                at: 0,
                resources: ResourceVector { values: [1] },
            },
            TraceSample {
                at: 1,
                resources: ResourceVector { values: [2] },
            },
        ],
        provenance_digest: TRACE_PROVENANCE,
    }
}

fn predictor_artifact() -> PredictorArtifact<1, 1> {
    PredictorArtifact {
        model_digest: MODEL_DIGEST,
        runtime_profile_digest: RUNTIME_PROFILE,
        axes: [ResourceAxisId(0)],
        horizons: [ForecastHorizon { start: 2, end: 3 }],
    }
}

fn forecast_matrix() -> ForecastMatrix<1, 1> {
    ForecastMatrix {
        subject_id: 21,
        axes: [ResourceAxisId(0)],
        horizons: [ForecastHorizon { start: 2, end: 3 }],
        cells: [[ForecastCell {
            expected: 3,
            lower: 2,
            upper: 4,
            confidence_ppm: 500_000,
        }]],
        observation_digest: trace().digest(),
        model_digest: MODEL_DIGEST,
        provenance_digest: TRACE_PROVENANCE,
    }
}

// non-vacuity: src/sa2a/predictive_model.rs:267/273/276 — deleting the
// observation/provenance digest or runtime-profile comparison makes the
// corresponding assert stop failing. (AxisMismatch) src/sa2a/predictive_model.rs:115
// — deleting the `matrix.axes[axis] != registry.axes[axis]` comparison (or
// seeding the registry from the matrix instead of the artifact) makes the
// first assert stop failing.
#[test]
fn forecast_matrix_admission_refuses_identity_and_axis_mismatches_exactly() {
    assert!(admit_forecast_matrix(
        &trace(),
        &predictor_artifact(),
        RUNTIME_PROFILE,
        forecast_matrix()
    )
    .is_ok());

    let wrong_observation = ForecastMatrix {
        observation_digest: 0,
        ..forecast_matrix()
    };
    assert_eq!(
        admit_forecast_matrix(
            &trace(),
            &predictor_artifact(),
            RUNTIME_PROFILE,
            wrong_observation
        ),
        Err(ForecastModelRefusal::ObservationDigestMismatch)
    );

    let wrong_provenance = ForecastMatrix {
        provenance_digest: 0,
        ..forecast_matrix()
    };
    assert_eq!(
        admit_forecast_matrix(
            &trace(),
            &predictor_artifact(),
            RUNTIME_PROFILE,
            wrong_provenance
        ),
        Err(ForecastModelRefusal::ProvenanceDigestMismatch)
    );

    assert_eq!(
        admit_forecast_matrix(
            &trace(),
            &predictor_artifact(),
            RUNTIME_PROFILE ^ 1,
            forecast_matrix()
        ),
        Err(ForecastModelRefusal::RuntimeProfileMismatch)
    );

    let wrong_axis = ForecastMatrix {
        axes: [ResourceAxisId(9)],
        ..forecast_matrix()
    };
    assert_eq!(
        admit_forecast_matrix(&trace(), &predictor_artifact(), RUNTIME_PROFILE, wrong_axis),
        Err(ForecastModelRefusal::AxisMismatch { index: 0 })
    );
}

// ---------------------------------------------------------------------------
// shadow_optimizer (src/sa2a/shadow_optimizer.rs) — module previously had ZERO
// exact-variant coverage.
// ---------------------------------------------------------------------------

fn population() -> [ShadowCandidate<2>; 4] {
    // Two clusters; lower fitness wins (ties break on lower candidate_id).
    // Champions: cluster 0 -> id 1 (fitness 10), cluster 1 -> id 3 (fitness 5);
    // global champion: id 3.
    [
        ShadowCandidate {
            candidate_id: 1,
            cluster: 0,
            model_digest: 100,
            fitness: 10,
            position: [0, 0],
        },
        ShadowCandidate {
            candidate_id: 2,
            cluster: 0,
            model_digest: 200,
            fitness: 20,
            position: [10, 0],
        },
        ShadowCandidate {
            candidate_id: 3,
            cluster: 1,
            model_digest: 300,
            fitness: 5,
            position: [0, 0],
        },
        ShadowCandidate {
            candidate_id: 4,
            cluster: 1,
            model_digest: 400,
            fitness: 15,
            position: [0, 4],
        },
    ]
}

// non-vacuity: src/sa2a/shadow_optimizer.rs:55-60 — swapping the C==0 and K==0
// checks (or deleting either) makes one of the first two asserts stop failing.
// :71-75 — widening the `candidate.cluster >= K` bound check (e.g. `> K`) or
// dropping it makes the third assert stop failing. :104-106 — initializing
// local_ids to 0 instead of u64::MAX (or deleting the MissingClusterChampion
// scan) makes the fourth assert stop failing.
#[test]
fn shadow_analyze_population_refuses_empty_and_shape_violations_exactly() {
    let empty: [ShadowCandidate<2>; 0] = [];
    assert_eq!(
        analyze_population::<0, 2, 2>(&empty),
        Err(ShadowOptimizerRefusal::NoCandidates)
    );

    assert_eq!(
        analyze_population::<4, 0, 2>(&population()),
        Err(ShadowOptimizerRefusal::NoClusters)
    );

    let mut stray = population();
    stray[3].cluster = 5; // >= K = 2
    assert_eq!(
        analyze_population::<4, 2, 2>(&stray),
        Err(ShadowOptimizerRefusal::ClusterOutOfRange {
            candidate_id: 4,
            cluster: 5
        })
    );

    let mut one_cluster = population();
    one_cluster[2].cluster = 0;
    one_cluster[3].cluster = 0; // cluster 1 now has no members
    assert_eq!(
        analyze_population::<4, 2, 2>(&one_cluster),
        Err(ShadowOptimizerRefusal::MissingClusterChampion { cluster: 1 })
    );
}

// non-vacuity: src/sa2a/shadow_optimizer.rs:158-164 — relaxing any single ppm
// bound (e.g. `>= 1_000_000` to `> 2_000_000`) makes the first assert stop
// failing. :312-324 (find_candidate) — making find_candidate return index 0 on
// miss instead of Err, or deleting the candidate_id scan, makes the second
// assert stop failing. The exact-assert on candidate_id 999 also fails if the
// error carries the wrong id.
#[test]
fn shadow_attraction_refuses_bad_probability_and_unknown_candidate_exactly() {
    assert_eq!(
        attraction_proposal(
            &population(),
            &analyze_population::<4, 2, 2>(&population()).unwrap(),
            2,
            1_000_001,
            0,
            0,
            0
        ),
        Err(ShadowOptimizerRefusal::InvalidProbabilityPpm)
    );

    assert_eq!(
        attraction_proposal(
            &population(),
            &analyze_population::<4, 2, 2>(&population()).unwrap(),
            999,
            1_000_000,
            500_000,
            1_000_000,
            250_000
        ),
        Err(ShadowOptimizerRefusal::CandidateNotFound { candidate_id: 999 })
    );
}

// Positive control for the shadow fixture: the refusal tests above must fail
// because of the injected violation, not because the fixture itself is broken.
// Exact arithmetic: candidate 2 at [10,0]; effective local factor
// (1e6 * 5e5) / 1e12 = 0.5 toward champion 1 at [0,0]; effective global factor
// (1e6 * 25e4) / 1e12 = 0.25 toward champion 3 at [0,0] => proposed[0] =
// 10 - 5 - 2 = 3 (i128 trunc division), proposed[1] = 0.
// non-vacuity: src/sa2a/shadow_optimizer.rs:187-202 — a mutation flipping the
// attraction delta sign, swapping local/global, or dividing by a different
// denominator changes the exact [3, 0] / [false, false, false, true] outputs.
#[test]
fn shadow_attraction_and_rejuvenation_happy_path_is_exact() {
    let analysis = analyze_population::<4, 2, 2>(&population()).unwrap();
    let proposed = attraction_proposal(
        &population(),
        &analysis,
        2,
        1_000_000,
        1_000_000,
        500_000,
        250_000,
    )
    .unwrap();
    assert_eq!(proposed, [3, 0]);

    let mask = shadow_rejuvenation_mask(&population(), &analysis, 5).unwrap();
    // Champions 1 and 3 never marked; candidate 2 at [10,0] sits 10 hops from
    // both champions (beyond horizon 5); candidate 4 at [0,4] sits 4 hops from
    // its local champion 3 / global champion 3 (within horizon 5).
    assert_eq!(mask, [false, false, false, true]);
}

// ---------------------------------------------------------------------------
// symbolic_envelope (src/sa2a/symbolic_envelope.rs)
// ---------------------------------------------------------------------------

fn family() -> SymbolicFamily<1> {
    SymbolicFamily {
        subject_id: 31,
        family_id: 0xF1,
        reachability_mask: 0b11,
        domains: [SymbolDomain {
            lower: 1,
            upper: 9,
            alignment: 2, // admits 1, 3, 5, 7, 9
        }],
        cpu: AffineResourceExpr {
            base: 10,
            coeffs: [2],
        },
        memory: AffineResourceExpr {
            base: 10,
            coeffs: [1],
        },
        io: AffineResourceExpr {
            base: 10,
            coeffs: [1],
        },
    }
}

fn witness(family_id: u64, value: u64) -> SymbolicWitness<1> {
    SymbolicWitness {
        family_id,
        values: [value],
    }
}

// non-vacuity: src/sa2a/symbolic_envelope.rs:124-129 — deleting the family-id
// comparison (or comparing witness.family_id against itself) makes the first
// assert stop failing. :134-136 — dropping the cardinality==0 guard makes the
// second assert stop failing. :137-141 — replacing domain.contains with `true`
// makes the third assert stop failing. :146-157 — replacing the checked
// arithmetic in AffineResourceExpr::evaluate (symbolic_envelope.rs:78-87) with
// wrapping arithmetic makes the fourth assert stop failing (Ok instead of Err).
#[test]
fn symbolic_concretize_refuses_identity_domain_and_overflow_exactly() {
    // Positive control: witness value 3 => cpu 10 + 2*3 = 16, memory/io 13.
    assert_eq!(
        family().concretize(&witness(0xF1, 3)),
        Ok(Allocation {
            cpu: 16,
            memory: 13,
            io: 13
        })
    );

    assert_eq!(
        family().concretize(&witness(0xF2, 3)),
        Err(SymbolicRefusal::FamilyIdMismatch {
            expected: 0xF1,
            actual: 0xF2
        })
    );

    let mut degenerate = family();
    degenerate.domains[0].alignment = 0; // cardinality 0
    assert_eq!(
        degenerate.concretize(&witness(0xF1, 3)),
        Err(SymbolicRefusal::InvalidDomain { index: 0 })
    );

    assert_eq!(
        family().concretize(&witness(0xF1, 4)), // in [1,9] but off-alignment
        Err(SymbolicRefusal::ValueOutOfDomain { index: 0, value: 4 })
    );

    let mut overflowing = family();
    overflowing.cpu.base = u64::MAX;
    assert_eq!(
        overflowing.concretize(&witness(0xF1, 3)),
        Err(SymbolicRefusal::ArithmeticOverflow)
    );
}

// non-vacuity: src/sa2a/symbolic_envelope.rs:262-266 — deleting the
// `start >= end` lifetime guard makes the assert stop failing (Ok(peak)
// instead of Err(InvalidLifetime { index: 1 })).
#[test]
fn peak_reusable_usage_refuses_degenerate_lifetime_exactly() {
    let timed = [
        TimedAllocation {
            start: 0,
            end: 10,
            allocation: Allocation {
                cpu: 1,
                memory: 1,
                io: 1,
            },
        },
        TimedAllocation {
            start: 5,
            end: 5, // degenerate: empty half-open lifetime
            allocation: Allocation {
                cpu: 1,
                memory: 1,
                io: 1,
            },
        },
    ];
    assert_eq!(
        peak_reusable_usage(&timed),
        Err(SymbolicRefusal::InvalidLifetime { index: 1 })
    );
}
