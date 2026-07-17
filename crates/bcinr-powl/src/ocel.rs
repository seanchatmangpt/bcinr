//! ocel — Object-Centric Event Log for POWL conformance checking.
//!
//! Records `op_fired` and `run_sealed` events into a fixed-capacity log,
//! enabling process-mining conformance checks without heap allocation.

#![forbid(unsafe_code)]

pub struct OcelEvent {
    pub event_id: u64,
    pub activity: &'static str, // "op_fired" | "run_sealed"
    pub timestamp: u64,         // monotonic tick counter
    pub run_id: u64,
    pub op_idx: u32,
    pub kind_tag: u8,
}

pub struct OcelLog {
    events: [OcelEvent; 512],
    count: usize,
    tick: u64,
}

impl OcelLog {
    pub const fn new() -> Self {
        const DEFAULT_EVENT: OcelEvent = OcelEvent {
            event_id: 0,
            activity: "",
            timestamp: 0,
            run_id: 0,
            op_idx: 0,
            kind_tag: 0,
        };
        Self {
            events: [DEFAULT_EVENT; 512],
            count: 0,
            tick: 0,
        }
    }

    /// Record that operation `op_idx` fired within `run_id`.
    pub fn record_op_fired(&mut self, run_id: u64, op_idx: u32, kind_tag: u8) {
        self.tick += 1;
        if self.count < 512 {
            self.events[self.count] = OcelEvent {
                event_id: self.count as u64,
                activity: "op_fired",
                timestamp: self.tick,
                run_id,
                op_idx,
                kind_tag,
            };
            self.count += 1;
        }
    }

    /// Record that run `run_id` was sealed with the given `op_trace` bitmask.
    /// The low 32 bits of `op_trace` are stored in `op_idx`; `kind_tag` is 0.
    pub fn record_run_sealed(&mut self, run_id: u64, op_trace: u64) {
        self.tick += 1;
        if self.count < 512 {
            self.events[self.count] = OcelEvent {
                event_id: self.count as u64,
                activity: "run_sealed",
                timestamp: self.tick,
                run_id,
                op_idx: op_trace as u32,
                kind_tag: 0,
            };
            self.count += 1;
        }
    }

    /// Return the slice of recorded events.
    pub fn events(&self) -> &[OcelEvent] {
        &self.events[..self.count]
    }
}

impl Default for OcelLog {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ocel_log_conforms_to_powl_model() {
        let mut log = OcelLog::new();
        let run_id = 42u64;
        log.record_op_fired(run_id, 0, 1);
        log.record_op_fired(run_id, 1, 2);
        log.record_run_sealed(run_id, 0b11);
        let events = log.events();
        let op_fired_runs: Vec<u64> = events.iter()
            .filter(|e| e.activity == "op_fired")
            .map(|e| e.run_id)
            .collect();
        let sealed_runs: Vec<u64> = events.iter()
            .filter(|e| e.activity == "run_sealed")
            .map(|e| e.run_id)
            .collect();
        for run in &op_fired_runs {
            assert!(sealed_runs.contains(run), "run {run} has op_fired but no run_sealed");
        }
        let sealed_ts = events.iter()
            .find(|e| e.activity == "run_sealed" && e.run_id == run_id)
            .map(|e| e.timestamp)
            .expect("run_sealed must exist");
        for e in events.iter().filter(|e| e.activity == "op_fired" && e.run_id == run_id) {
            assert!(e.timestamp < sealed_ts, "op_fired at {} must precede run_sealed at {}", e.timestamp, sealed_ts);
        }
        let op_idxs: Vec<u32> = events.iter()
            .filter(|e| e.activity == "op_fired" && e.run_id == run_id)
            .map(|e| e.op_idx)
            .collect();
        let mut seen = std::collections::HashSet::new();
        for idx in &op_idxs {
            assert!(seen.insert(idx), "duplicate op_idx {idx} in run {run_id}");
        }
        let computed_trace: u64 = op_idxs.iter().fold(0u64, |acc, &idx| acc | (1u64 << idx));
        let sealed_trace = events.iter()
            .find(|e| e.activity == "run_sealed" && e.run_id == run_id)
            .map(|e| e.op_idx as u64)
            .expect("run_sealed must exist");
        assert_eq!(computed_trace, sealed_trace, "op_trace mismatch: computed {computed_trace:#b} vs sealed {sealed_trace:#b}");
    }

    #[test]
    fn ocel_rejects_impossible_op_trace() {
        let mut log = OcelLog::new();
        let run_id = 99u64;
        log.record_op_fired(run_id, 0, 1);
        log.record_op_fired(run_id, 1, 2);
        log.record_run_sealed(run_id, 0b111);
        let events = log.events();
        let op_fired_count = events.iter()
            .filter(|e| e.activity == "op_fired" && e.run_id == run_id)
            .count();
        let sealed_trace = events.iter()
            .find(|e| e.activity == "run_sealed" && e.run_id == run_id)
            .map(|e| e.op_idx as u64)
            .expect("run_sealed must exist");
        let sealed_op_count = sealed_trace.count_ones() as usize;
        assert!(op_fired_count < sealed_op_count,
            "Expected impossible trace gap: op_fired count ({op_fired_count}) < op_trace.count_ones() ({sealed_op_count})");
    }
}
