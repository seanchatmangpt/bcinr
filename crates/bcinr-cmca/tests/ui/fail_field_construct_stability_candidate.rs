use bcinr_cmca::stability::StabilityCandidate;
fn main() {
    let _ = StabilityCandidate {
        g: [[0i64, 0i64], [0i64, 0i64]],
        d: [0i64, 0i64],
        margin_delta: 0,
        noise_radius: 0,
        switch_radius: 0,
        q_ceiling: 0,
        gram_distinguishability_floor: 0,
        dwell_law_id: 0,
        pricing_loop_bound: 0,
        comparison_derivation_identity: 0,
        candidate_digest: 0,
    };
}
