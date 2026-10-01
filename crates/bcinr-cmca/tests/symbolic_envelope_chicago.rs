use bcinr_cmca::sa2a::{
    peak_reusable_usage, pipeline_latency, AffineResourceExpr, Allocation, PipelineKnobs,
    ResourceEnvelope, SymbolDomain, SymbolicFamily, SymbolicRefusal, SymbolicWitness,
    TimedAllocation,
};

fn family2() -> SymbolicFamily<2> {
    SymbolicFamily {
        subject_id: 7,
        family_id: 42,
        reachability_mask: 0b0111,
        domains: [
            SymbolDomain {
                lower: 1,
                upper: 8,
                alignment: 1,
            },
            SymbolDomain {
                lower: 16,
                upper: 64,
                alignment: 16,
            },
        ],
        cpu: AffineResourceExpr {
            base: 0,
            coeffs: [1, 0],
        },
        memory: AffineResourceExpr {
            base: 0,
            coeffs: [0, 2],
        },
        io: AffineResourceExpr {
            base: 5,
            coeffs: [0, 0],
        },
    }
}

#[test]
fn chicago_symbolic_family_admits_a_concrete_witness_without_selection_authority() {
    let family = family2();
    let envelope = ResourceEnvelope {
        cpu: 4,
        memory: 64,
        io: 5,
    };
    let witness = SymbolicWitness {
        family_id: 42,
        values: [4, 32],
    };

    assert_eq!(
        family.admit_in(&envelope, &witness),
        Ok(Allocation {
            cpu: 4,
            memory: 64,
            io: 5,
        })
    );
}

#[test]
fn chicago_symbolic_family_refuses_out_of_domain_and_out_of_envelope_witnesses() {
    let family = family2();
    let envelope = ResourceEnvelope {
        cpu: 4,
        memory: 64,
        io: 5,
    };

    assert_eq!(
        family.admit_in(
            &envelope,
            &SymbolicWitness {
                family_id: 42,
                values: [4, 30],
            },
        ),
        Err(SymbolicRefusal::ValueOutOfDomain {
            index: 1,
            value: 30,
        })
    );

    assert_eq!(
        family.admit_in(
            &envelope,
            &SymbolicWitness {
                family_id: 42,
                values: [5, 32],
            },
        ),
        Err(SymbolicRefusal::EnvelopeExceeded {
            requested: Allocation {
                cpu: 5,
                memory: 64,
                io: 5,
            },
            envelope,
        })
    );
}

#[test]
fn chicago_symbolic_membership_matches_small_exhaustive_court() {
    let family = SymbolicFamily {
        subject_id: 1,
        family_id: 9,
        reachability_mask: 1,
        domains: [
            SymbolDomain {
                lower: 1,
                upper: 3,
                alignment: 1,
            },
            SymbolDomain {
                lower: 2,
                upper: 4,
                alignment: 2,
            },
        ],
        cpu: AffineResourceExpr {
            base: 0,
            coeffs: [1, 0],
        },
        memory: AffineResourceExpr {
            base: 0,
            coeffs: [0, 1],
        },
        io: AffineResourceExpr {
            base: 0,
            coeffs: [0, 0],
        },
    };
    let envelope = ResourceEnvelope {
        cpu: 100,
        memory: 100,
        io: 100,
    };

    let mut left = 0u64;
    while left <= 5 {
        let mut right = 0u64;
        while right <= 5 {
            let witness = SymbolicWitness {
                family_id: 9,
                values: [left, right],
            };
            let expected =
                family.domains[0].contains(left) && family.domains[1].contains(right);
            assert_eq!(
                family.admit_in(&envelope, &witness).is_ok(),
                expected,
                "membership drift for values [{left}, {right}]"
            );
            right += 1;
        }
        left += 1;
    }

    assert_eq!(family.concrete_space_saturating(), 6);
}

#[test]
fn symbolic_family_represents_large_cross_product_without_enumeration() {
    let family = SymbolicFamily {
        subject_id: 1,
        family_id: 10,
        reachability_mask: 1,
        domains: [SymbolDomain {
            lower: 1,
            upper: 32,
            alignment: 1,
        }; 4],
        cpu: AffineResourceExpr {
            base: 0,
            coeffs: [1, 0, 0, 0],
        },
        memory: AffineResourceExpr {
            base: 0,
            coeffs: [0, 1, 0, 0],
        },
        io: AffineResourceExpr {
            base: 0,
            coeffs: [0, 0, 1, 0],
        },
    };

    assert_eq!(family.concrete_space_saturating(), 1_048_576);
}

#[test]
fn chicago_subsumption_prunes_only_when_options_are_preserved() {
    let expr = AffineResourceExpr {
        base: 0,
        coeffs: [1, 1],
    };
    let wide = SymbolicFamily {
        subject_id: 77,
        family_id: 1,
        reachability_mask: 0b0111,
        domains: [
            SymbolDomain {
                lower: 1,
                upper: 8,
                alignment: 1,
            },
            SymbolDomain {
                lower: 1,
                upper: 8,
                alignment: 1,
            },
        ],
        cpu: expr,
        memory: expr,
        io: expr,
    };
    let narrow = SymbolicFamily {
        subject_id: 77,
        family_id: 2,
        reachability_mask: 0b0011,
        domains: [
            SymbolDomain {
                lower: 2,
                upper: 6,
                alignment: 2,
            },
            SymbolDomain {
                lower: 4,
                upper: 4,
                alignment: 99,
            },
        ],
        cpu: expr,
        memory: expr,
        io: expr,
    };

    assert!(wide.subsumes_without_option_loss(&narrow));

    let reachability_loss = SymbolicFamily {
        reachability_mask: 0b1000,
        ..narrow
    };
    assert!(!wide.subsumes_without_option_loss(&reachability_loss));

    let wrong_subject = SymbolicFamily {
        subject_id: 78,
        ..narrow
    };
    assert!(!wide.subsumes_without_option_loss(&wrong_subject));
}

#[test]
fn chicago_lifetime_reuse_counts_physical_peak_not_logical_sum() {
    let non_overlapping = [
        TimedAllocation {
            start: 0,
            end: 10,
            allocation: Allocation {
                cpu: 10,
                memory: 20,
                io: 1,
            },
        },
        TimedAllocation {
            start: 10,
            end: 20,
            allocation: Allocation {
                cpu: 10,
                memory: 20,
                io: 1,
            },
        },
    ];
    assert_eq!(
        peak_reusable_usage(&non_overlapping),
        Ok(Allocation {
            cpu: 10,
            memory: 20,
            io: 1,
        })
    );

    let overlapping = [
        non_overlapping[0],
        TimedAllocation {
            start: 9,
            ..non_overlapping[1]
        },
    ];
    assert_eq!(
        peak_reusable_usage(&overlapping),
        Ok(Allocation {
            cpu: 20,
            memory: 40,
            io: 2,
        })
    );
}

#[test]
fn chicago_pipeline_latency_matches_loom_heg_equations() {
    assert_eq!(
        pipeline_latency(
            3,
            5,
            2,
            PipelineKnobs {
                load_body_open: false,
                body_store_open: false,
            },
        ),
        Ok(10)
    );
    assert_eq!(
        pipeline_latency(
            3,
            5,
            2,
            PipelineKnobs {
                load_body_open: false,
                body_store_open: true,
            },
        ),
        Ok(8)
    );
    assert_eq!(
        pipeline_latency(
            3,
            5,
            2,
            PipelineKnobs {
                load_body_open: true,
                body_store_open: false,
            },
        ),
        Ok(7)
    );
    assert_eq!(
        pipeline_latency(
            3,
            5,
            2,
            PipelineKnobs {
                load_body_open: true,
                body_store_open: true,
            },
        ),
        Ok(5)
    );
}
