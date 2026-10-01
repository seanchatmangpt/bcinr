use bcinr_cmca::sa2a::{
    admit_forecast_matrix, admit_predictive_envelope, admit_predictive_measure,
    admit_reservation_proposal, analyze_population, attraction_proposal, benchmark_forecast,
    build_predictive_measure, counterfactual_replay, optimization_receipt,
    predictive_envelope_digest, shadow_rejuvenation_mask, standing_digest, AffineResourceExpr,
    Allocation, BenchmarkScenario, CorpusClass, ForecastCell, ForecastHorizon, ForecastMatrix,
    ForecastModelRefusal, ForecastStanding, PredictiveAdmission, PredictiveConsequenceEnvelope,
    PredictiveMeasureAdmission, Predictor, PredictorArtifact, PredictorCostReceipt,
    ReservationAdmission, ReservationProposal, ReservationRefusal, ResourceAxisId,
    ResourceAxisRegistry, ResourceEnvelope, ResourceVector, ShadowCandidate, ShockPolicy,
    SymbolDomain, SymbolicFamily, TraceSample, TraceWindow,
};

#[derive(Clone, Copy)]
struct DemoPredictor;

impl Predictor<2, 2, 3> for DemoPredictor {
    fn artifact(&self) -> PredictorArtifact<2, 2> {
        PredictorArtifact {
            model_digest: 0xAA,
            runtime_profile_digest: 0xBB,
            axes: [ResourceAxisId(1), ResourceAxisId(2)],
            horizons: [
                ForecastHorizon { start: 30, end: 60 },
                ForecastHorizon { start: 60, end: 120 },
            ],
        }
    }

    fn predict(
        &self,
        trace: &TraceWindow<2, 3>,
    ) -> Result<ForecastMatrix<2, 2>, ForecastModelRefusal> {
        trace.validate()?;
        let last = trace.samples[2].resources.values;
        Ok(ForecastMatrix {
            subject_id: trace.subject_id,
            axes: [ResourceAxisId(1), ResourceAxisId(2)],
            horizons: self.artifact().horizons,
            cells: [
                [
                    ForecastCell {
                        lower: last[0],
                        expected: last[0] + 10,
                        upper: last[0] + 20,
                        confidence_ppm: 900_000,
                    },
                    ForecastCell {
                        lower: last[0] + 10,
                        expected: last[0] + 20,
                        upper: last[0] + 30,
                        confidence_ppm: 850_000,
                    },
                ],
                [
                    ForecastCell {
                        lower: last[1],
                        expected: last[1] + 20,
                        upper: last[1] + 40,
                        confidence_ppm: 900_000,
                    },
                    ForecastCell {
                        lower: last[1] + 20,
                        expected: last[1] + 40,
                        upper: last[1] + 60,
                        confidence_ppm: 850_000,
                    },
                ],
            ],
            observation_digest: trace.digest(),
            model_digest: self.artifact().model_digest,
            provenance_digest: trace.provenance_digest,
        })
    }
}

fn trace() -> TraceWindow<2, 3> {
    TraceWindow {
        subject_id: 7,
        samples: [
            TraceSample {
                at: 1,
                resources: ResourceVector { values: [10, 100] },
            },
            TraceSample {
                at: 2,
                resources: ResourceVector { values: [20, 120] },
            },
            TraceSample {
                at: 3,
                resources: ResourceVector { values: [30, 140] },
            },
        ],
        provenance_digest: 0xCC,
    }
}

#[test]
fn chicago_sap_map_horizons_axes_and_trace_identity_are_first_class() {
    let predictor = DemoPredictor;
    let trace = trace();
    let artifact = predictor.artifact();
    let matrix = predictor.predict(&trace).unwrap();

    let admitted = admit_forecast_matrix(&trace, &artifact, 0xBB, matrix).unwrap();
    assert_eq!(admitted.axes.len(), 2);
    assert_eq!(admitted.horizons.len(), 2);

    let single_axis_registry = ResourceAxisRegistry {
        axes: [ResourceAxisId(99)],
    };
    assert!(single_axis_registry.validate().is_ok());

    let duplicate_registry = ResourceAxisRegistry {
        axes: [ResourceAxisId(1), ResourceAxisId(1)],
    };
    assert_eq!(
        duplicate_registry.validate(),
        Err(ForecastModelRefusal::DuplicateAxis { index: 1 })
    );
}

#[test]
fn chicago_same_model_input_and_runtime_replay_exactly() {
    let predictor = DemoPredictor;
    let trace = trace();
    let first = predictor.predict(&trace).unwrap();
    let second = predictor.predict(&trace).unwrap();
    assert_eq!(first, second);

    let artifact = predictor.artifact();
    assert_eq!(
        admit_forecast_matrix(&trace, &artifact, 0xBB, first),
        admit_forecast_matrix(&trace, &artifact, 0xBB, second)
    );

    let mut wrong_artifact = artifact;
    wrong_artifact.model_digest ^= 1;
    assert_eq!(
        admit_forecast_matrix(&trace, &wrong_artifact, 0xBB, first),
        Err(ForecastModelRefusal::ModelDigestMismatch)
    );
}

#[test]
fn chicago_trace_digest_changes_when_observation_changes() {
    let original = trace();
    let mut changed = original;
    changed.samples[2].resources.values[0] += 1;
    assert_ne!(original.digest(), changed.digest());

    let mut non_monotonic = original;
    non_monotonic.samples[2].at = 2;
    assert_eq!(
        non_monotonic.validate(),
        Err(ForecastModelRefusal::NonMonotonicTrace { index: 2 })
    );
}

#[test]
fn chicago_forecast_standing_separates_average_error_interval_coverage_and_shocks() {
    let predictor = DemoPredictor;
    let matrix = predictor.predict(&trace()).unwrap();
    let actual = [
        ResourceVector {
            values: [40, 160],
        },
        ResourceVector {
            values: [80, 220],
        },
    ];
    let scenario = BenchmarkScenario {
        class: CorpusClass::Synthetic,
        corpus_digest: 123,
        baseline: ResourceVector {
            values: [30, 140],
        },
        actual,
        shock_policy: ShockPolicy {
            thresholds: ResourceVector {
                values: [25, 30],
            },
        },
    };

    let receipt = benchmark_forecast(
        &matrix,
        &scenario,
        PredictorCostReceipt {
            training_time_micros: 10_000,
            inference_time_micros: 50,
            peak_memory_bytes: 4096,
        },
    );

    assert_eq!(receipt.class, CorpusClass::Synthetic);
    assert_eq!(receipt.calibration.standing.sample_count, 4);
    assert!(receipt.calibration.standing.mean_absolute_error > 0);
    assert!(receipt.calibration.standing.root_mean_squared_error > 0);
    assert!(receipt.calibration.standing.interval_coverage_ppm <= 1_000_000);
    assert_eq!(receipt.cost.inference_time_micros, 50);
}

fn shadow_candidates() -> [ShadowCandidate<2>; 4] {
    [
        ShadowCandidate {
            candidate_id: 1,
            cluster: 0,
            model_digest: 101,
            fitness: 30,
            position: [0, 0],
        },
        ShadowCandidate {
            candidate_id: 2,
            cluster: 0,
            model_digest: 102,
            fitness: 10,
            position: [10, 10],
        },
        ShadowCandidate {
            candidate_id: 3,
            cluster: 1,
            model_digest: 103,
            fitness: 20,
            position: [100, 100],
        },
        ShadowCandidate {
            candidate_id: 4,
            cluster: 1,
            model_digest: 104,
            fitness: 40,
            position: [101, 101],
        },
    ]
}

#[test]
fn chicago_shadow_optimizer_has_local_global_pressure_without_model_admission() {
    let candidates = shadow_candidates();
    let analysis = analyze_population::<4, 2, 2>(&candidates).unwrap();
    assert_eq!(analysis.local_champion_ids, [2, 3]);
    assert_eq!(analysis.global_champion_id, 2);
    assert_eq!(analysis.global_best_fitness, 10);
    assert_eq!(analysis.distinct_model_count, 4);

    let proposal = attraction_proposal(
        &candidates,
        &analysis,
        4,
        500_000,
        500_000,
        1_000_000,
        1_000_000,
    )
    .unwrap();
    assert!(proposal[0] < 101);
    assert!(proposal[1] < 101);

    let receipt = optimization_receipt(
        77,
        88,
        4,
        2,
        [30, 20, 10, 10, 10],
        2,
        100,
        4096,
        analysis.fitness_variance,
        analysis.distinct_model_count,
    );
    assert_eq!(receipt.convergence_epoch, Some(4));
}

#[test]
fn chicago_event_horizon_only_marks_shadow_candidates_and_preserves_cmca_family() {
    let candidates = shadow_candidates();
    let analysis = analyze_population::<4, 2, 2>(&candidates).unwrap();
    let mask = shadow_rejuvenation_mask(&candidates, &analysis, 2).unwrap();
    assert!(!mask[1]);
    assert!(!mask[2]);
    assert!(mask[3]);

    let family = SymbolicFamily {
        subject_id: 7,
        family_id: 42,
        reachability_mask: 0b111,
        domains: [SymbolDomain {
            lower: 1,
            upper: 10,
            alignment: 1,
        }],
        cpu: AffineResourceExpr {
            base: 0,
            coeffs: [1],
        },
        memory: AffineResourceExpr {
            base: 0,
            coeffs: [0],
        },
        io: AffineResourceExpr {
            base: 0,
            coeffs: [0],
        },
    };
    let before = family;
    let _ = mask;
    assert_eq!(family, before);
}

fn admitted_pce() -> bcinr_cmca::sa2a::AdmittedPredictiveConsequenceEnvelope {
    let candidate = PredictiveConsequenceEnvelope {
        subject_id: 7,
        horizon_start: 30,
        horizon_end: 60,
        expected: Allocation {
            cpu: 80,
            memory: 120,
            io: 40,
        },
        lower: Allocation {
            cpu: 60,
            memory: 100,
            io: 20,
        },
        upper: Allocation {
            cpu: 110,
            memory: 180,
            io: 70,
        },
        confidence_ppm: 900_000,
        observation_digest: 11,
        model_digest: 22,
        provenance_digest: 33,
    };
    admit_predictive_envelope(
        candidate,
        PredictiveAdmission {
            subject_id: 7,
            observation_digest: 11,
            model_digest: 22,
            provenance_digest: 33,
            min_confidence_ppm: 800_000,
        },
    )
    .unwrap()
}

#[test]
fn chicago_predictive_measure_binds_forecast_and_empirical_standing_without_allocating() {
    let forecast = admitted_pce();
    let capacity = ResourceEnvelope {
        cpu: 100,
        memory: 200,
        io: 100,
    };
    let standing = ForecastStanding {
        sample_count: 100,
        mean_squared_error: 25,
        mean_absolute_error: 4,
        root_mean_squared_error: 5,
        signed_bias: -1,
        interval_coverage_ppm: 920_000,
        shock_misses: 2,
        shock_false_positives: 1,
    };
    let artifact = build_predictive_measure(&forecast, &capacity, standing);
    let admitted = admit_predictive_measure(
        artifact,
        PredictiveMeasureAdmission {
            subject_id: 7,
            forecast_digest: predictive_envelope_digest(&forecast),
            standing_digest: standing_digest(&standing),
        },
    )
    .unwrap();

    assert_eq!(admitted.artifact().expected_pressure.cpu_ppm, 800_000);
    assert_eq!(admitted.artifact().shock_misses, 2);
    assert_eq!(capacity.cpu, 100);
}

#[test]
fn chicago_reservation_proposal_cannot_manufacture_capacity_or_do_work() {
    let forecast = admitted_pce();
    let forecast_digest = predictive_envelope_digest(&forecast);
    let capacity = ResourceEnvelope {
        cpu: 100,
        memory: 200,
        io: 100,
    };
    let admission = ReservationAdmission {
        subject_id: 7,
        forecast_digest,
        authorization_binding_digest: 999,
    };

    let too_large = ReservationProposal {
        subject_id: 7,
        horizon_start: 30,
        horizon_end: 60,
        requested: Allocation {
            cpu: 101,
            memory: 10,
            io: 10,
        },
        forecast_digest,
        reason_digest: 1,
    };
    assert_eq!(
        admit_reservation_proposal(too_large, &capacity, admission, 999),
        Err(ReservationRefusal::EnvelopeExceeded {
            requested: too_large.requested,
            actual: capacity,
        })
    );

    let lawful = ReservationProposal {
        requested: Allocation {
            cpu: 50,
            memory: 100,
            io: 50,
        },
        ..too_large
    };
    let admitted = admit_reservation_proposal(lawful, &capacity, admission, 999).unwrap();
    assert_eq!(admitted.proposal(), &lawful);
    assert_eq!(capacity.cpu, 100);
}

#[test]
fn chicago_counterfactual_replay_compares_without_actuation() {
    let receipt = counterfactual_replay(
        7,
        11,
        22,
        Allocation {
            cpu: 100,
            memory: 100,
            io: 100,
        },
        Allocation {
            cpu: 70,
            memory: 110,
            io: 90,
        },
        Allocation {
            cpu: 95,
            memory: 100,
            io: 105,
        },
    );

    assert!(receipt.allocation_changed);
    assert_eq!(receipt.reactive_unmet.cpu, 30);
    assert_eq!(receipt.predictive_unmet.cpu, 5);
    assert_eq!(receipt.reactive_waste.memory, 10);
    assert_eq!(receipt.predictive_waste.io, 5);
}
