use bcinr_cmca::sa2a::{
    admit_predictive_envelope, AffineResourceExpr, Allocation, PredictiveAdmission,
    PredictiveConsequenceEnvelope, PredictiveRefusal, PressureVector, ResourceAxis,
    ResourceEnvelope, SymbolDomain, SymbolicFamily, SymbolicRefusal, SymbolicWitness,
    PRESSURE_PPM_ONE,
};

fn forecast() -> PredictiveConsequenceEnvelope {
    PredictiveConsequenceEnvelope {
        subject_id: 7,
        horizon_start: 100,
        horizon_end: 105,
        expected: Allocation {
            cpu: 80,
            memory: 100,
            io: 25,
        },
        lower: Allocation {
            cpu: 60,
            memory: 80,
            io: 20,
        },
        upper: Allocation {
            cpu: 120,
            memory: 250,
            io: 60,
        },
        confidence_ppm: 900_000,
        observation_digest: 11,
        model_digest: 22,
        provenance_digest: 33,
    }
}

fn admission() -> PredictiveAdmission {
    PredictiveAdmission {
        subject_id: 7,
        observation_digest: 11,
        model_digest: 22,
        provenance_digest: 33,
        min_confidence_ppm: 800_000,
    }
}

#[test]
fn chicago_predictive_envelope_admits_exact_identity_and_reports_bounded_pressure() {
    let admitted = admit_predictive_envelope(forecast(), admission()).unwrap();
    let actual = ResourceEnvelope {
        cpu: 100,
        memory: 200,
        io: 50,
    };

    let pressure = admitted.pressure_against(&actual);
    assert_eq!(
        pressure.expected,
        PressureVector {
            cpu_ppm: 800_000,
            memory_ppm: 500_000,
            io_ppm: 500_000,
        }
    );
    assert_eq!(
        pressure.upper,
        PressureVector {
            cpu_ppm: PRESSURE_PPM_ONE,
            memory_ppm: PRESSURE_PPM_ONE,
            io_ppm: PRESSURE_PPM_ONE,
        }
    );
}

#[test]
fn chicago_forecast_can_predict_overload_without_manufacturing_budget() {
    let mut candidate = forecast();
    candidate.expected = Allocation {
        cpu: 1_000,
        memory: 1_000,
        io: 1_000,
    };
    candidate.upper = Allocation {
        cpu: 2_000,
        memory: 2_000,
        io: 2_000,
    };
    candidate.lower = Allocation {
        cpu: 900,
        memory: 900,
        io: 900,
    };

    let admitted = admit_predictive_envelope(candidate, admission()).unwrap();
    let actual = ResourceEnvelope {
        cpu: 10,
        memory: 10,
        io: 10,
    };
    assert_eq!(
        admitted.pressure_against(&actual).upper,
        PressureVector {
            cpu_ppm: PRESSURE_PPM_ONE,
            memory_ppm: PRESSURE_PPM_ONE,
            io_ppm: PRESSURE_PPM_ONE,
        }
    );

    let family = SymbolicFamily {
        subject_id: 7,
        family_id: 9,
        reachability_mask: 1,
        domains: [SymbolDomain {
            lower: 11,
            upper: 11,
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
    let witness = SymbolicWitness {
        family_id: 9,
        values: [11],
    };

    assert_eq!(
        family.admit_in(&actual, &witness),
        Err(SymbolicRefusal::EnvelopeExceeded {
            requested: Allocation {
                cpu: 11,
                memory: 0,
                io: 0,
            },
            envelope: actual,
        })
    );
}

#[test]
fn chicago_predictive_admission_refuses_identity_confidence_and_interval_drift() {
    let mut wrong_subject = forecast();
    wrong_subject.subject_id = 8;
    assert_eq!(
        admit_predictive_envelope(wrong_subject, admission()),
        Err(PredictiveRefusal::SubjectMismatch {
            expected: 7,
            actual: 8,
        })
    );

    let mut wrong_model = forecast();
    wrong_model.model_digest = 99;
    assert_eq!(
        admit_predictive_envelope(wrong_model, admission()),
        Err(PredictiveRefusal::ModelDigestMismatch)
    );

    let mut low_confidence = forecast();
    low_confidence.confidence_ppm = 799_999;
    assert_eq!(
        admit_predictive_envelope(low_confidence, admission()),
        Err(PredictiveRefusal::ConfidenceBelowFloor {
            floor_ppm: 800_000,
            actual_ppm: 799_999,
        })
    );

    let mut invalid_bounds = forecast();
    invalid_bounds.lower.memory = 101;
    assert_eq!(
        admit_predictive_envelope(invalid_bounds, admission()),
        Err(PredictiveRefusal::LowerExceedsExpected {
            axis: ResourceAxis::Memory,
        })
    );
}

#[test]
fn chicago_predictive_admission_refuses_invalid_horizon_and_confidence_domain() {
    let mut invalid_horizon = forecast();
    invalid_horizon.horizon_end = invalid_horizon.horizon_start;
    assert_eq!(
        admit_predictive_envelope(invalid_horizon, admission()),
        Err(PredictiveRefusal::InvalidHorizon)
    );

    let mut invalid_confidence = forecast();
    invalid_confidence.confidence_ppm = PRESSURE_PPM_ONE + 1;
    assert_eq!(
        admit_predictive_envelope(invalid_confidence, admission()),
        Err(PredictiveRefusal::InvalidConfidence {
            confidence_ppm: PRESSURE_PPM_ONE + 1,
        })
    );

    let mut invalid_floor = admission();
    invalid_floor.min_confidence_ppm = PRESSURE_PPM_ONE + 1;
    assert_eq!(
        admit_predictive_envelope(forecast(), invalid_floor),
        Err(PredictiveRefusal::InvalidConfidenceFloor {
            floor_ppm: PRESSURE_PPM_ONE + 1,
        })
    );
}

#[test]
fn chicago_forecast_receipt_falsifies_interval_and_records_absolute_error() {
    let admitted = admit_predictive_envelope(forecast(), admission()).unwrap();

    let inside = admitted.observe(Allocation {
        cpu: 90,
        memory: 120,
        io: 30,
    });
    assert!(inside.within_interval);
    assert_eq!(
        inside.absolute_error,
        Allocation {
            cpu: 10,
            memory: 20,
            io: 5,
        }
    );
    assert_eq!(inside.model_digest, 22);
    assert_eq!(inside.provenance_digest, 33);

    let outside = admitted.observe(Allocation {
        cpu: 121,
        memory: 120,
        io: 30,
    });
    assert!(!outside.within_interval);
}

#[test]
fn chicago_predictive_boundary_is_deterministic_under_replay() {
    let admitted_a = admit_predictive_envelope(forecast(), admission()).unwrap();
    let admitted_b = admit_predictive_envelope(forecast(), admission()).unwrap();
    let capacity = ResourceEnvelope {
        cpu: u64::MAX,
        memory: 0,
        io: 1,
    };
    let observation = Allocation {
        cpu: 77,
        memory: 88,
        io: 99,
    };

    assert_eq!(
        admitted_a.pressure_against(&capacity),
        admitted_b.pressure_against(&capacity)
    );
    assert_eq!(
        admitted_a.observe(observation),
        admitted_b.observe(observation)
    );
}
