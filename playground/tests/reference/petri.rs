//! Plain-Rust Petri-net reference implementation, mirroring
//! `wasm4pm-compat`'s `Place`/`Transition`/`Arc`/`Marking` model, for
//! differential-testing `playground`'s branchless Petri primitives against.
//!
//! `#![allow(dead_code)]`: this is a comprehensive reference surface, not a
//! single-purpose fixture — it is compiled independently into several
//! separate test binaries via `mod reference;` (`reference_test.rs`,
//! `differential_tests.rs`, …), and each binary only exercises the subset of
//! this API its own tests actually call. A method unused by one binary's
//! compilation is routinely used by another's; item-by-item allows would
//! just be noise repeated ~20 times over, not a more honest signal.
#![allow(dead_code)]

use std::collections::HashMap;

// ── Types and Models from wasm4pm-compat ────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Place {
    pub id: String,
}

impl Place {
    pub fn new(id: &str) -> Self {
        Place { id: id.to_owned() }
    }
    pub fn id(&self) -> &str {
        &self.id
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transition {
    pub id: String,
    pub label: String,
    pub is_invisible: Option<bool>,
}

impl Transition {
    pub fn new(id: &str, label: &str) -> Self {
        Transition { id: id.to_owned(), label: label.to_owned(), is_invisible: Some(false) }
    }
    pub fn silent(id: &str) -> Self {
        Transition { id: id.to_owned(), label: String::new(), is_invisible: Some(true) }
    }
    pub fn id(&self) -> &str {
        &self.id
    }
    pub fn label(&self) -> &str {
        &self.label
    }
    pub fn is_silent(&self) -> bool {
        self.is_invisible.unwrap_or(false)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Arc {
    pub from: String,
    pub to: String,
    pub weight: Option<usize>,
    pub object_type: Option<(String, bool)>,
}

impl Arc {
    pub fn place_to_transition(place: &str, transition: &str) -> Self {
        Arc {
            from: place.to_owned(),
            to: transition.to_owned(),
            weight: Some(1),
            object_type: None,
        }
    }
    pub fn transition_to_place(transition: &str, place: &str) -> Self {
        Arc {
            from: transition.to_owned(),
            to: place.to_owned(),
            weight: Some(1),
            object_type: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SoundnessState {
    Unknown,
    Claimed,
    Witnessed,
}

#[derive(Debug, Clone, Default)]
pub struct Marking {
    tokens: Vec<(String, usize)>,
}

impl Marking {
    pub fn new(tokens: impl IntoIterator<Item = (String, usize)>) -> Self {
        Marking { tokens: tokens.into_iter().collect() }
    }
    pub fn empty() -> Self {
        Marking::default()
    }
    pub fn is_empty(&self) -> bool {
        self.tokens.is_empty()
    }
    pub fn tokens(&self) -> &[(String, usize)] {
        &self.tokens
    }
    pub fn tokens_on(&self, place_id: &str) -> usize {
        self.tokens.iter().find(|(id, _)| id == place_id).map_or(0, |(_, n)| *n)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PetriRefusal {
    MissingInitialMarking,
    MissingFinalMarking,
    DeadTransition,
    UnsafeNet,
    ObjectTypeNotPreserved,
    SoundnessNotWitnessed,
    InvalidInstanceBounds,
}

#[derive(Debug, Clone)]
pub struct PetriNet {
    pub places: Vec<Place>,
    pub transitions: Vec<Transition>,
    pub arcs: Vec<Arc>,
    pub initial_marking: Vec<(String, usize)>,
    pub final_marking: Vec<(String, usize)>,
}

impl PetriNet {
    pub fn new(
        places: impl IntoIterator<Item = Place>,
        transitions: impl IntoIterator<Item = Transition>,
        arcs: impl IntoIterator<Item = Arc>,
        initial_marking: Marking,
    ) -> Self {
        PetriNet {
            places: places.into_iter().collect(),
            transitions: transitions.into_iter().collect(),
            arcs: arcs.into_iter().collect(),
            initial_marking: initial_marking.tokens.clone(),
            final_marking: Vec::new(),
        }
    }
}

pub const SOUNDNESS_UNKNOWN: u8 = 0;
pub const SOUNDNESS_CLAIMED: u8 = 1;
pub const SOUNDNESS_WITNESSED: u8 = 2;

pub struct WfNetConst<const S: u8> {
    pub net: PetriNet,
    pub final_marking: Marking,
}

impl WfNetConst<SOUNDNESS_UNKNOWN> {
    pub fn new(net: PetriNet, final_marking: Marking) -> Self {
        Self { net, final_marking }
    }
    pub fn claim_sound(self) -> WfNetConst<SOUNDNESS_CLAIMED> {
        WfNetConst { net: self.net, final_marking: self.final_marking }
    }
}

impl WfNetConst<SOUNDNESS_CLAIMED> {
    pub fn witness_soundness(self) -> WfNetConst<SOUNDNESS_WITNESSED> {
        WfNetConst { net: self.net, final_marking: self.final_marking }
    }
}

impl<const S: u8> WfNetConst<S> {
    pub fn soundness_state(&self) -> SoundnessState {
        match S {
            SOUNDNESS_CLAIMED => SoundnessState::Claimed,
            SOUNDNESS_WITNESSED => SoundnessState::Witnessed,
            _ => SoundnessState::Unknown,
        }
    }
    pub fn validate(&self) -> Result<(), PetriRefusal> {
        if self.final_marking.is_empty() {
            return Err(PetriRefusal::MissingFinalMarking);
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct ObjectCentricPetriNet {
    pub net: PetriNet,
    pub object_types: Vec<String>,
}

impl ObjectCentricPetriNet {
    pub fn new(net: PetriNet, object_types: impl IntoIterator<Item = String>) -> Self {
        Self { net, object_types: object_types.into_iter().collect() }
    }
    pub fn validate(&self) -> Result<(), PetriRefusal> {
        let type_set: std::collections::HashSet<&str> =
            self.object_types.iter().map(std::string::String::as_str).collect();
        for arc in &self.net.arcs {
            if let Some((ref ot, _)) = arc.object_type {
                if !type_set.contains(ot.as_str()) {
                    return Err(PetriRefusal::ObjectTypeNotPreserved);
                }
            }
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum InstanceCreationKind {
    Static,
    Dynamic,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MultipleInstanceSpec {
    pub min: u32,
    pub max: Option<u32>,
    pub threshold: Option<u32>,
    pub creation: InstanceCreationKind,
}

impl MultipleInstanceSpec {
    pub fn new(
        min: u32,
        max: Option<u32>,
        threshold: Option<u32>,
        creation: InstanceCreationKind,
    ) -> Self {
        MultipleInstanceSpec { min, max, threshold, creation }
    }
    pub fn validate(&self) -> Result<(), PetriRefusal> {
        if self.min == 0 {
            return Err(PetriRefusal::InvalidInstanceBounds);
        }
        if let Some(max) = self.max {
            if self.min > max {
                return Err(PetriRefusal::InvalidInstanceBounds);
            }
        }
        Ok(())
    }
}

// ── Replay Engine Types from dteam ──────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReplayResult {
    pub missing: u32,
    pub remaining: u32,
    pub produced: u32,
    pub consumed: u32,
}

impl ReplayResult {
    pub fn fitness(&self) -> f64 {
        let total = self.consumed + self.missing;
        if total == 0 && self.produced == 0 {
            return 1.0;
        }
        let denom = f64::from(total + self.produced);
        if denom == 0.0 {
            return 1.0;
        }
        1.0 - (f64::from(self.missing) + f64::from(self.remaining)) / denom
    }
    pub fn is_perfect(&self) -> bool {
        self.missing == 0 && self.remaining == 0
    }
}

pub struct TransMask {
    pub in_mask: u64,
    pub out_mask: u64,
    pub is_invisible: bool,
    pub in_popcount: u32,
    pub out_popcount: u32,
}

pub struct NetBitmask64 {
    pub initial_mask: u64,
    pub final_mask: u64,
    pub n_places: usize,
    pub transitions: Vec<TransMask>,
    pub label_index: Vec<(String, Vec<usize>)>,
    pub invisible_indices: Vec<usize>,
}

impl NetBitmask64 {
    pub fn from_petri_net(net: &PetriNet) -> Self {
        let n_places = net.places.len();
        assert!(n_places <= 64, "NetBitmask64 requires <=64 places");

        let mut place_bit = HashMap::with_capacity(n_places);
        for (i, p) in net.places.iter().enumerate() {
            place_bit.insert(p.id.as_str(), 1u64 << i);
        }

        let mut initial_mask = 0u64;
        for (p_id, count) in &net.initial_marking {
            if *count > 0 {
                if let Some(&bit) = place_bit.get(p_id.as_str()) {
                    initial_mask |= bit;
                }
            }
        }

        let mut final_mask = 0u64;
        for (p_id, count) in &net.final_marking {
            if *count > 0 {
                if let Some(&bit) = place_bit.get(p_id.as_str()) {
                    final_mask |= bit;
                }
            }
        }

        let n_trans = net.transitions.len();
        let mut in_masks = vec![0u64; n_trans];
        let mut out_masks = vec![0u64; n_trans];

        let mut trans_idx = HashMap::with_capacity(n_trans);
        for (i, t) in net.transitions.iter().enumerate() {
            trans_idx.insert(t.id.as_str(), i);
        }

        for arc in &net.arcs {
            if let Some(&ti) = trans_idx.get(arc.to.as_str()) {
                if let Some(&bit) = place_bit.get(arc.from.as_str()) {
                    in_masks[ti] |= bit;
                }
            } else if let Some(&ti) = trans_idx.get(arc.from.as_str()) {
                if let Some(&bit) = place_bit.get(arc.to.as_str()) {
                    out_masks[ti] |= bit;
                }
            }
        }

        let transitions: Vec<TransMask> = net
            .transitions
            .iter()
            .enumerate()
            .map(|(i, t)| {
                let in_mask = in_masks[i];
                let out_mask = out_masks[i];
                let is_invisible = t.is_invisible.unwrap_or(false)
                    || t.label.starts_with('$')
                    || t.label.is_empty();
                TransMask {
                    in_mask,
                    out_mask,
                    is_invisible,
                    in_popcount: in_mask.count_ones(),
                    out_popcount: out_mask.count_ones(),
                }
            })
            .collect();

        let mut label_map: HashMap<String, Vec<usize>> = HashMap::new();
        for (i, t) in net.transitions.iter().enumerate() {
            if !transitions[i].is_invisible {
                label_map.entry(t.label.clone()).or_default().push(i);
            }
        }
        let mut label_index: Vec<(String, Vec<usize>)> = label_map.into_iter().collect();
        label_index.sort_by(|a, b| a.0.cmp(&b.0));

        let invisible_indices: Vec<usize> = transitions
            .iter()
            .enumerate()
            .filter(|(_, t)| t.is_invisible)
            .map(|(i, _)| i)
            .collect();

        Self { initial_mask, final_mask, n_places, transitions, label_index, invisible_indices }
    }
}

// ── Event Log Types ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AttributeValue {
    String(String),
    Int(i64),
}

#[derive(Debug, Clone)]
pub struct Attribute {
    pub key: String,
    pub value: AttributeValue,
}

#[derive(Debug, Clone)]
pub struct Event {
    pub attributes: Vec<Attribute>,
}

#[derive(Debug, Clone)]
pub struct Trace {
    pub id: String,
    pub attributes: Vec<Attribute>,
    pub events: Vec<Event>,
}

#[derive(Debug, Clone)]
pub struct EventLog {
    pub traces: Vec<Trace>,
}

// ── Replay logic functions ──────────────────────────────────────────────────

fn fire_invisible(net: &NetBitmask64, marking: &mut u64) {
    let mut changed = true;
    while changed {
        changed = false;
        for &i in &net.invisible_indices {
            let t = &net.transitions[i];
            if (*marking & t.in_mask) == t.in_mask {
                *marking = (*marking & !t.in_mask) | t.out_mask;
                changed = true;
                break;
            }
        }
    }
}

pub fn replay_trace(net: &NetBitmask64, trace: &Trace) -> ReplayResult {
    let mut marking = net.initial_mask;
    let mut missing: u32 = 0;
    let mut consumed: u32 = 0;
    let mut produced: u32 = net.initial_mask.count_ones();

    fire_invisible(net, &mut marking);

    for event in &trace.events {
        let activity = event.attributes.iter().find(|a| a.key == "concept:name").and_then(|a| {
            if let AttributeValue::String(s) = &a.value {
                Some(s.as_str())
            } else {
                None
            }
        });

        let Some(activity) = activity else { continue };

        let t_indices = match net.label_index.binary_search_by(|(k, _)| k.as_str().cmp(activity)) {
            Ok(pos) => &net.label_index[pos].1,
            Err(_) => continue,
        };

        let t_idx = t_indices
            .iter()
            .copied()
            .find(|&i| (marking & net.transitions[i].in_mask) == net.transitions[i].in_mask)
            .unwrap_or(t_indices[0]);

        let t = &net.transitions[t_idx];

        let need = t.in_mask & !marking;
        if need != 0 {
            missing += need.count_ones();
            marking |= need;
        }

        marking = (marking & !t.in_mask) | t.out_mask;
        consumed += t.in_popcount;
        produced += t.out_popcount;

        fire_invisible(net, &mut marking);
    }

    let final_needed = net.final_mask.count_ones();
    let final_have = (marking & net.final_mask).count_ones();
    if final_needed > final_have {
        missing += final_needed - final_have;
        marking |= net.final_mask & !marking;
    }
    consumed += final_needed;
    marking &= !net.final_mask;
    let remaining = marking.count_ones();

    ReplayResult { missing, remaining, produced, consumed }
}

pub fn replay_log(net: &NetBitmask64, log: &EventLog) -> Vec<ReplayResult> {
    log.traces.iter().map(|t| replay_trace(net, t)).collect()
}

struct MarkingSet {
    inline: [u64; 64],
    len: usize,
}

impl MarkingSet {
    fn new_with(start: u64) -> Self {
        let mut inline = [0u64; 64];
        inline[0] = start;
        Self { inline, len: 1 }
    }
    fn new_empty() -> Self {
        Self { inline: [0u64; 64], len: 0 }
    }
    fn contains(&self, m: u64) -> bool {
        self.inline[..self.len].contains(&m)
    }
    fn push(&mut self, m: u64) {
        if self.len < 64 {
            self.inline[self.len] = m;
            self.len += 1;
        }
    }
    fn len(&self) -> usize {
        self.len
    }
    fn is_empty(&self) -> bool {
        self.len == 0
    }
    fn get(&self, i: usize) -> u64 {
        self.inline[i]
    }
    fn iter_slice(&self) -> &[u64] {
        &self.inline[..self.len]
    }
}

fn epsilon_close(net: &NetBitmask64, start: u64) -> MarkingSet {
    let mut reachable = MarkingSet::new_with(start);
    let mut i = 0;
    while i < reachable.len() {
        let m = reachable.get(i);
        for &ti in &net.invisible_indices {
            let t = &net.transitions[ti];
            if (m & t.in_mask) == t.in_mask {
                let new_m = (m & !t.in_mask) | t.out_mask;
                if !reachable.contains(new_m) {
                    reachable.push(new_m);
                }
            }
        }
        i += 1;
    }
    reachable
}

pub fn in_language(net: &NetBitmask64, trace: &Trace) -> bool {
    let mut markings = epsilon_close(net, net.initial_mask);

    for event in &trace.events {
        let activity = event.attributes.iter().find(|a| a.key == "concept:name").and_then(|a| {
            if let AttributeValue::String(s) = &a.value {
                Some(s.as_str())
            } else {
                None
            }
        });
        let Some(activity) = activity else { continue };

        let t_indices = match net.label_index.binary_search_by(|(k, _)| k.as_str().cmp(activity)) {
            Ok(pos) => &net.label_index[pos].1,
            Err(_) => continue,
        };

        let mut next = MarkingSet::new_empty();
        for &m in markings.iter_slice() {
            for &ti in t_indices {
                let t = &net.transitions[ti];
                if (m & t.in_mask) == t.in_mask {
                    let new_m = (m & !t.in_mask) | t.out_mask;
                    let eps = epsilon_close(net, new_m);
                    for em in eps.iter_slice() {
                        if !next.contains(*em) {
                            next.push(*em);
                        }
                    }
                }
            }
        }

        if next.is_empty() {
            return false;
        }
        markings = next;
    }

    markings.iter_slice().iter().any(|&m| (m & net.final_mask) == net.final_mask)
}

pub fn count_in_language(net: &NetBitmask64, log: &EventLog) -> usize {
    log.traces.iter().filter(|t| in_language(net, t)).count()
}

pub fn classify_exact(net: &NetBitmask64, log: &EventLog, n_target: usize) -> Vec<bool> {
    let in_lang: Vec<bool> = log.traces.iter().map(|t| in_language(net, t)).collect();
    let n_accepted = in_lang.iter().filter(|&&b| b).count();

    if n_accepted == n_target {
        return in_lang;
    }

    let results = replay_log(net, log);
    let mut accepted: Vec<(usize, f64)> = in_lang
        .iter()
        .enumerate()
        .filter(|(_, &b)| b)
        .map(|(i, _)| (i, results[i].fitness()))
        .collect();
    let mut rejected: Vec<(usize, f64)> = in_lang
        .iter()
        .enumerate()
        .filter(|(_, &b)| !b)
        .map(|(i, _)| (i, results[i].fitness()))
        .collect();

    accepted.sort_by(|a, b| {
        b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal).then(a.0.cmp(&b.0))
    });
    rejected.sort_by(|a, b| {
        b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal).then(a.0.cmp(&b.0))
    });

    let mut out = vec![false; log.traces.len()];
    let take_accepted = n_accepted.min(n_target);
    for &(i, _) in &accepted[..take_accepted] {
        out[i] = true;
    }
    let fill = n_target.saturating_sub(take_accepted);
    for &(i, _) in rejected.iter().take(fill) {
        out[i] = true;
    }
    out
}

pub fn classify(results: &[ReplayResult], n_target: usize) -> Vec<bool> {
    let mut perfect: Vec<usize> = Vec::new();
    let mut imperfect: Vec<(usize, f64)> = Vec::new();

    for (i, r) in results.iter().enumerate() {
        if r.is_perfect() {
            perfect.push(i);
        } else {
            imperfect.push((i, r.fitness()));
        }
    }

    imperfect.sort_by(|a, b| {
        b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal).then(a.0.cmp(&b.0))
    });

    let mut out = vec![false; results.len()];

    if perfect.len() >= n_target {
        perfect.sort_unstable();
        for &i in &perfect[..n_target] {
            out[i] = true;
        }
    } else {
        for &i in &perfect {
            out[i] = true;
        }
        let fill = n_target.saturating_sub(perfect.len());
        for &(i, _) in imperfect.iter().take(fill) {
            out[i] = true;
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_soundness_claims() {
        let net = PetriNet {
            places: vec![Place::new("p1")],
            transitions: vec![],
            arcs: vec![],
            initial_marking: vec![("p1".to_string(), 1)],
            final_marking: vec![("p1".to_string(), 1)],
        };
        let wf = WfNetConst::<SOUNDNESS_UNKNOWN>::new(net, Marking::new([("p1".to_string(), 1)]));
        assert_eq!(wf.soundness_state(), SoundnessState::Unknown);

        let claimed = wf.claim_sound();
        assert_eq!(claimed.soundness_state(), SoundnessState::Claimed);

        let witnessed = claimed.witness_soundness();
        assert_eq!(witnessed.soundness_state(), SoundnessState::Witnessed);
    }
}
