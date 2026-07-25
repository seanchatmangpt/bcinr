/// Residual reconciliation input. This utility decides whether an existing
/// suffix remains eligible for verification; it does not pretend to manufacture
/// a replacement plan without invoking a semantic rail.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResidualRequest {
    pub original_plan: PlanRoot,
    pub original_observation: ObservationRoot,
    pub current_observation: ObservationRoot,
    pub next_tick: Option<u32>,
    pub goal_already_satisfied: bool,
    pub generation: u32,
    pub max_generations: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReplanDecision {
    KeepSuffix {
        from_tick: Option<u32>,
    },
    ReplaceRequired {
        previous_observation: ObservationRoot,
        current_observation: ObservationRoot,
    },
    GoalAlreadySatisfied,
    Refuse {
        reason: ReplanRefusal,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReplanRefusal {
    GenerationBoundExceeded { limit: u32 },
}

pub fn reconcile_residual(request: &ResidualRequest) -> ReplanDecision {
    if request.generation >= request.max_generations {
        return ReplanDecision::Refuse {
            reason: ReplanRefusal::GenerationBoundExceeded {
                limit: request.max_generations,
            },
        };
    }
    if request.goal_already_satisfied {
        return ReplanDecision::GoalAlreadySatisfied;
    }
    if request.original_observation == request.current_observation {
        ReplanDecision::KeepSuffix {
            from_tick: request.next_tick,
        }
    } else {
        ReplanDecision::ReplaceRequired {
            previous_observation: request.original_observation,
            current_observation: request.current_observation,
        }
    }
}
