//! Execution Substrate: Staged plans, cells, and resumable stream states.

/// A trait for kernels that can be executed as a staged pipeline stage.
pub(crate) trait PipelineStage {
    type Input;
    type Output;
    type State;

    fn execute(&self, input: &Self::Input, state: &mut Self::State, output: &mut Self::Output);
}

/// A resumable cell for streaming data processing.
pub(crate) struct ExecutionCell<S: PipelineStage> {
    pub stage: S,
    pub state: S::State,
}

impl<S: PipelineStage> ExecutionCell<S> {
    pub fn new(stage: S, state: S::State) -> Self {
        Self { stage, state }
    }

    pub fn process(&mut self, input: &S::Input, output: &mut S::Output) {
        self.stage.execute(input, &mut self.state, output);
    }
}

/// A simple execution plan for an edge confidence kernel.
pub(crate) struct EdgeConfidencePlan {
    pub activity_count: usize,
}

impl PipelineStage for EdgeConfidencePlan {
    type Input = (u16, u16); // (from, to)
    type Output = u32;      // new confidence
    type State = Vec<u32>;  // dense edge field

    fn execute(&self, input: &Self::Input, state: &mut Self::State, output: &mut Self::Output) {
        let (from, to) = *input;
        let idx = (from as usize) * self.activity_count + (to as usize);
        debug_assert!(idx < state.len(), "EdgeConfidencePlan: index {idx} out of bounds (state len {})", state.len());
        state[idx] = state[idx].saturating_add(1);
        *output = state[idx];
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_edge_confidence_cell() {
        let plan = EdgeConfidencePlan { activity_count: 10 };
        let state = vec![0u32; 100];
        let mut cell = ExecutionCell::new(plan, state);

        let mut out = 0u32;
        cell.process(&(1, 2), &mut out);
        assert_eq!(out, 1);
        cell.process(&(1, 2), &mut out);
        assert_eq!(out, 2);
    }
}
