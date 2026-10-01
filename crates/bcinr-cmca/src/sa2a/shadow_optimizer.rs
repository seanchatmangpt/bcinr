//! Shadow-only population optimization evidence.
//!
//! The local/global champion topology is inspired by QB-BiO, but this module
//! has no authority to admit a model, mutate CMCA state, or delete lawful
//! options. Event-horizon logic can only mark ephemeral shadow candidates for
//! rejuvenation.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ShadowCandidate<const D: usize> {
    pub candidate_id: u64,
    pub cluster: usize,
    pub model_digest: u64,
    pub fitness: u64,
    pub position: [i64; D],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PopulationAnalysis<const K: usize> {
    pub local_champion_ids: [u64; K],
    pub global_champion_id: u64,
    pub global_best_fitness: u64,
    pub fitness_variance: u64,
    pub distinct_model_count: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ShadowOptimizerRefusal {
    NoCandidates,
    NoClusters,
    ClusterOutOfRange {
        candidate_id: u64,
        cluster: usize,
    },
    MissingClusterChampion {
        cluster: usize,
    },
    CandidateNotFound {
        candidate_id: u64,
    },
    InvalidProbabilityPpm,
}

impl core::fmt::Display for ShadowOptimizerRefusal {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Debug::fmt(self, f)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for ShadowOptimizerRefusal {}

pub fn analyze_population<const C: usize, const K: usize, const D: usize>(
    candidates: &[ShadowCandidate<D>; C],
) -> Result<PopulationAnalysis<K>, ShadowOptimizerRefusal> {
    if C == 0 {
        return Err(ShadowOptimizerRefusal::NoCandidates);
    }
    if K == 0 {
        return Err(ShadowOptimizerRefusal::NoClusters);
    }

    let mut local_ids = [u64::MAX; K];
    let mut local_fitness = [u64::MAX; K];
    let mut global_id = u64::MAX;
    let mut global_fitness = u64::MAX;
    let mut fitness_sum = 0u128;

    let mut i = 0usize;
    while i < C {
        let candidate = candidates[i];
        if candidate.cluster >= K {
            return Err(ShadowOptimizerRefusal::ClusterOutOfRange {
                candidate_id: candidate.candidate_id,
                cluster: candidate.cluster,
            });
        }

        if better(
            candidate.fitness,
            candidate.candidate_id,
            local_fitness[candidate.cluster],
            local_ids[candidate.cluster],
        ) {
            local_fitness[candidate.cluster] = candidate.fitness;
            local_ids[candidate.cluster] = candidate.candidate_id;
        }

        if better(
            candidate.fitness,
            candidate.candidate_id,
            global_fitness,
            global_id,
        ) {
            global_fitness = candidate.fitness;
            global_id = candidate.candidate_id;
        }

        fitness_sum = fitness_sum.saturating_add(candidate.fitness as u128);
        i += 1;
    }

    let mut cluster = 0usize;
    while cluster < K {
        if local_ids[cluster] == u64::MAX {
            return Err(ShadowOptimizerRefusal::MissingClusterChampion { cluster });
        }
        cluster += 1;
    }

    let mean = fitness_sum / C as u128;
    let mut variance_sum = 0u128;
    let mut distinct_models = 0u64;

    i = 0;
    while i < C {
        let fitness = candidates[i].fitness as u128;
        let diff = fitness.abs_diff(mean);
        variance_sum = variance_sum.saturating_add(diff.saturating_mul(diff));

        let mut seen = false;
        let mut j = 0usize;
        while j < i {
            if candidates[j].model_digest == candidates[i].model_digest {
                seen = true;
                break;
            }
            j += 1;
        }
        if !seen {
            distinct_models = distinct_models.saturating_add(1);
        }
        i += 1;
    }

    Ok(PopulationAnalysis {
        local_champion_ids: local_ids,
        global_champion_id: global_id,
        global_best_fitness: global_fitness,
        fitness_variance: saturating_u128_to_u64(variance_sum / C as u128),
        distinct_model_count: distinct_models,
    })
}

/// Compute one QB-BiO-style local/global attraction proposal.
///
/// Random coefficients, if desired, are supplied explicitly as ppm values by
/// the shadow caller. This function itself is deterministic and powerless.
#[allow(clippy::too_many_arguments)]
pub fn attraction_proposal<const C: usize, const K: usize, const D: usize>(
    candidates: &[ShadowCandidate<D>; C],
    analysis: &PopulationAnalysis<K>,
    candidate_id: u64,
    local_attraction_ppm: u32,
    global_attraction_ppm: u32,
    local_draw_ppm: u32,
    global_draw_ppm: u32,
) -> Result<[i64; D], ShadowOptimizerRefusal> {
    if local_attraction_ppm > 1_000_000
        || global_attraction_ppm > 1_000_000
        || local_draw_ppm > 1_000_000
        || global_draw_ppm > 1_000_000
    {
        return Err(ShadowOptimizerRefusal::InvalidProbabilityPpm);
    }

    let candidate_index = find_candidate(candidates, candidate_id)?;
    let candidate = candidates[candidate_index];
    if candidate.cluster >= K {
        return Err(ShadowOptimizerRefusal::ClusterOutOfRange {
            candidate_id,
            cluster: candidate.cluster,
        });
    }

    let local_id = analysis.local_champion_ids[candidate.cluster];
    let local_index = find_candidate(candidates, local_id)?;
    let global_index = find_candidate(candidates, analysis.global_champion_id)?;
    let local = candidates[local_index];
    let global = candidates[global_index];

    let local_scale =
        (local_attraction_ppm as i128) * (local_draw_ppm as i128);
    let global_scale =
        (global_attraction_ppm as i128) * (global_draw_ppm as i128);
    let denominator = 1_000_000i128 * 1_000_000i128;

    let mut proposed = [0i64; D];
    let mut dimension = 0usize;
    while dimension < D {
        let current = candidate.position[dimension] as i128;
        let local_delta =
            (local.position[dimension] as i128 - current).saturating_mul(local_scale)
                / denominator;
        let global_delta =
            (global.position[dimension] as i128 - current).saturating_mul(global_scale)
                / denominator;
        proposed[dimension] =
            saturating_i128_to_i64(current.saturating_add(local_delta).saturating_add(global_delta));
        dimension += 1;
    }

    Ok(proposed)
}

/// Mark shadow candidates that are within an event-horizon distance of either
/// their local or global champion. Champions themselves are never marked.
///
/// The returned mask is only a recommendation to a shadow trainer to reseed
/// candidate models. No CMCA option or SymbolicFamily is accepted by this API.
pub fn shadow_rejuvenation_mask<const C: usize, const K: usize, const D: usize>(
    candidates: &[ShadowCandidate<D>; C],
    analysis: &PopulationAnalysis<K>,
    event_horizon: u64,
) -> Result<[bool; C], ShadowOptimizerRefusal> {
    let global_index = find_candidate(candidates, analysis.global_champion_id)?;
    let global = candidates[global_index];
    let mut mask = [false; C];

    let mut i = 0usize;
    while i < C {
        let candidate = candidates[i];
        if candidate.cluster >= K {
            return Err(ShadowOptimizerRefusal::ClusterOutOfRange {
                candidate_id: candidate.candidate_id,
                cluster: candidate.cluster,
            });
        }

        let local_id = analysis.local_champion_ids[candidate.cluster];
        let local_index = find_candidate(candidates, local_id)?;
        let local = candidates[local_index];

        let is_champion =
            candidate.candidate_id == local_id || candidate.candidate_id == global.candidate_id;
        if !is_champion {
            let local_distance = manhattan_distance(candidate.position, local.position);
            let global_distance = manhattan_distance(candidate.position, global.position);
            mask[i] = local_distance <= event_horizon || global_distance <= event_horizon;
        }
        i += 1;
    }

    Ok(mask)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OptimizationReceipt<const E: usize> {
    pub seed: u64,
    pub corpus_digest: u64,
    pub candidate_count: u64,
    pub cluster_count: u64,
    pub fitness_history: [u64; E],
    pub convergence_epoch: Option<usize>,
    pub wall_time_micros: u64,
    pub peak_memory_bytes: u64,
    pub final_population_variance: u64,
    pub final_distinct_model_count: u64,
}

/// Convergence is an observed plateau receipt, not a proof of global optimality.
#[allow(clippy::too_many_arguments)]
pub fn optimization_receipt<const E: usize>(
    seed: u64,
    corpus_digest: u64,
    candidate_count: u64,
    cluster_count: u64,
    fitness_history: [u64; E],
    plateau_patience: usize,
    wall_time_micros: u64,
    peak_memory_bytes: u64,
    final_population_variance: u64,
    final_distinct_model_count: u64,
) -> OptimizationReceipt<E> {
    let convergence_epoch = first_plateau(&fitness_history, plateau_patience);
    OptimizationReceipt {
        seed,
        corpus_digest,
        candidate_count,
        cluster_count,
        fitness_history,
        convergence_epoch,
        wall_time_micros,
        peak_memory_bytes,
        final_population_variance,
        final_distinct_model_count,
    }
}

fn first_plateau<const E: usize>(history: &[u64; E], patience: usize) -> Option<usize> {
    if E == 0 || patience == 0 {
        return None;
    }

    let mut best = history[0];
    let mut stagnant = 0usize;
    let mut epoch = 1usize;
    while epoch < E {
        if history[epoch] < best {
            best = history[epoch];
            stagnant = 0;
        } else {
            stagnant = stagnant.saturating_add(1);
            if stagnant >= patience {
                return Some(epoch);
            }
        }
        epoch += 1;
    }
    None
}

fn find_candidate<const C: usize, const D: usize>(
    candidates: &[ShadowCandidate<D>; C],
    candidate_id: u64,
) -> Result<usize, ShadowOptimizerRefusal> {
    let mut i = 0usize;
    while i < C {
        if candidates[i].candidate_id == candidate_id {
            return Ok(i);
        }
        i += 1;
    }
    Err(ShadowOptimizerRefusal::CandidateNotFound { candidate_id })
}

fn manhattan_distance<const D: usize>(left: [i64; D], right: [i64; D]) -> u64 {
    let mut sum = 0u128;
    let mut i = 0usize;
    while i < D {
        sum = sum.saturating_add((left[i] as i128).abs_diff(right[i] as i128));
        i += 1;
    }
    saturating_u128_to_u64(sum)
}

const fn better(
    fitness: u64,
    candidate_id: u64,
    incumbent_fitness: u64,
    incumbent_id: u64,
) -> bool {
    fitness < incumbent_fitness
        || (fitness == incumbent_fitness && candidate_id < incumbent_id)
}

fn saturating_u128_to_u64(value: u128) -> u64 {
    core::cmp::min(value, u64::MAX as u128) as u64
}

fn saturating_i128_to_i64(value: i128) -> i64 {
    if value > i64::MAX as i128 {
        i64::MAX
    } else if value < i64::MIN as i128 {
        i64::MIN
    } else {
        value as i64
    }
}
