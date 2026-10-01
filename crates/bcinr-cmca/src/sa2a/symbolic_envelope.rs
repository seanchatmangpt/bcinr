//! Symbolic resource envelopes for CMCA's authority-free SA2A boundary.
//!
//! This module applies the transferable part of Wang et al., "Schedules Are
//! Solvable Symbols" (arXiv:2609.29219): keep high-cardinality value choices
//! symbolic inside a discrete candidate family, derive/check legality against
//! explicit resource bounds, and only materialize a concrete witness after a
//! solver has chosen values.
//!
//! CMCA deliberately does not import Loom's final minimum-latency selection
//! authority. A symbolic witness is analytical/CONSTRUCT-only: this module can
//! validate it against a ResourceEnvelope, but cannot SELECT a family or DO.
//!
//! The representation is solver-agnostic. CP-SAT, SMT, a planner, or a human
//! can propose SymbolicWitness values; CMCA rechecks exact domains and resource
//! constraints before admitting the concretization.

use super::resource_envelope::{Allocation, ResourceEnvelope};

/// One source-traceable symbolic integer domain.
///
/// Values form the finite arithmetic progression lower, lower + alignment, ...
/// <= upper. An alignment of zero, or lower > upper, is invalid and refused.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SymbolDomain {
    pub lower: u64,
    pub upper: u64,
    pub alignment: u64,
}

impl SymbolDomain {
    pub const fn contains(&self, value: u64) -> bool {
        self.alignment != 0
            && self.lower <= self.upper
            && value >= self.lower
            && value <= self.upper
            && (value - self.lower) % self.alignment == 0
    }

    pub const fn cardinality(&self) -> u64 {
        if self.alignment == 0 || self.lower > self.upper {
            0
        } else {
            ((self.upper - self.lower) / self.alignment) + 1
        }
    }

    /// True when every concrete value in other is represented by self.
    pub const fn contains_domain(&self, other: &Self) -> bool {
        if self.alignment == 0
            || other.alignment == 0
            || self.lower > self.upper
            || other.lower > other.upper
            || other.lower < self.lower
            || other.upper > self.upper
            || !self.contains(other.lower)
            || !self.contains(other.upper)
        {
            return false;
        }

        if other.lower == other.upper {
            return true;
        }

        other.alignment % self.alignment == 0
    }
}

/// Non-negative affine resource expression:
/// base + SUM(coeffs[i] * symbol[i]).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AffineResourceExpr<const V: usize> {
    pub base: u64,
    pub coeffs: [u64; V],
}

impl<const V: usize> AffineResourceExpr<V> {
    pub fn evaluate(&self, values: &[u64; V]) -> Option<u64> {
        let mut total = self.base;
        let mut index = 0usize;
        while index < V {
            let term = self.coeffs[index].checked_mul(values[index])?;
            total = total.checked_add(term)?;
            index += 1;
        }
        Some(total)
    }
}

/// One discrete CMCA candidate family whose high-cardinality value choices
/// remain symbolic.
///
/// subject_id prevents cross-subject pruning. reachability_mask is an
/// option-preservation witness: a family may subsume another only when it
/// preserves every reachable edge represented by the narrower family.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SymbolicFamily<const V: usize> {
    pub subject_id: u64,
    pub family_id: u64,
    pub reachability_mask: u64,
    pub domains: [SymbolDomain; V],
    pub cpu: AffineResourceExpr<V>,
    pub memory: AffineResourceExpr<V>,
    pub io: AffineResourceExpr<V>,
}

impl<const V: usize> SymbolicFamily<V> {
    /// Number of concrete tuples represented without enumerating them.
    pub fn concrete_space_saturating(&self) -> u64 {
        let mut product = 1u64;
        let mut index = 0usize;
        while index < V {
            product = product.saturating_mul(self.domains[index].cardinality());
            index += 1;
        }
        product
    }

    /// Materialize and independently validate a solver-proposed witness.
    pub fn concretize(
        &self,
        witness: &SymbolicWitness<V>,
    ) -> Result<Allocation, SymbolicRefusal> {
        if witness.family_id != self.family_id {
            return Err(SymbolicRefusal::FamilyIdMismatch {
                expected: self.family_id,
                actual: witness.family_id,
            });
        }

        let mut index = 0usize;
        while index < V {
            let domain = self.domains[index];
            if domain.cardinality() == 0 {
                return Err(SymbolicRefusal::InvalidDomain { index });
            }
            if !domain.contains(witness.values[index]) {
                return Err(SymbolicRefusal::ValueOutOfDomain {
                    index,
                    value: witness.values[index],
                });
            }
            index += 1;
        }

        let cpu = self
            .cpu
            .evaluate(&witness.values)
            .ok_or(SymbolicRefusal::ArithmeticOverflow)?;
        let memory = self
            .memory
            .evaluate(&witness.values)
            .ok_or(SymbolicRefusal::ArithmeticOverflow)?;
        let io = self
            .io
            .evaluate(&witness.values)
            .ok_or(SymbolicRefusal::ArithmeticOverflow)?;

        Ok(Allocation { cpu, memory, io })
    }

    /// Admit a concretization only when it remains inside the explicit CMCA
    /// resource envelope.
    pub fn admit_in(
        &self,
        envelope: &ResourceEnvelope,
        witness: &SymbolicWitness<V>,
    ) -> Result<Allocation, SymbolicRefusal> {
        let allocation = self.concretize(witness)?;
        if envelope.admits(&allocation) {
            Ok(allocation)
        } else {
            Err(SymbolicRefusal::EnvelopeExceeded {
                requested: allocation,
                envelope: *envelope,
            })
        }
    }

    /// DfCM-safe pruning by exact set containment, not heuristic ranking.
    ///
    /// self can replace other only when both describe the exact same subject,
    /// self preserves all reachability, resource semantics are identical, and
    /// every symbolic domain in other is a subset of self.
    pub fn subsumes_without_option_loss(&self, other: &Self) -> bool {
        if self.subject_id != other.subject_id
            || (self.reachability_mask | other.reachability_mask) != self.reachability_mask
            || self.cpu != other.cpu
            || self.memory != other.memory
            || self.io != other.io
        {
            return false;
        }

        let mut index = 0usize;
        while index < V {
            if !self.domains[index].contains_domain(&other.domains[index]) {
                return false;
            }
            index += 1;
        }

        true
    }
}

/// Concrete values proposed for a symbolic family.
///
/// Possession of a witness is not authority: it must pass admit_in and still
/// carries no SELECT/DO capability.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SymbolicWitness<const V: usize> {
    pub family_id: u64,
    pub values: [u64; V],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SymbolicRefusal {
    FamilyIdMismatch { expected: u64, actual: u64 },
    InvalidDomain { index: usize },
    ValueOutOfDomain { index: usize, value: u64 },
    ArithmeticOverflow,
    EnvelopeExceeded {
        requested: Allocation,
        envelope: ResourceEnvelope,
    },
    InvalidLifetime { index: usize },
}

impl core::fmt::Display for SymbolicRefusal {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Debug::fmt(self, f)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for SymbolicRefusal {}

/// Logical resource use over a half-open lifetime [start, end).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TimedAllocation {
    pub start: u64,
    pub end: u64,
    pub allocation: Allocation,
}

fn checked_add_allocation(left: Allocation, right: Allocation) -> Option<Allocation> {
    Some(Allocation {
        cpu: left.cpu.checked_add(right.cpu)?,
        memory: left.memory.checked_add(right.memory)?,
        io: left.io.checked_add(right.io)?,
    })
}

/// Compute physical peak usage when non-overlapping logical lifetimes can
/// reuse capacity. The active set can only increase at a start point, so
/// probing starts is sufficient. Bounded O(M^2), stack-only.
pub fn peak_reusable_usage<const M: usize>(
    allocations: &[TimedAllocation; M],
) -> Result<Allocation, SymbolicRefusal> {
    let mut index = 0usize;
    while index < M {
        if allocations[index].start >= allocations[index].end {
            return Err(SymbolicRefusal::InvalidLifetime { index });
        }
        index += 1;
    }

    let mut peak = Allocation {
        cpu: 0,
        memory: 0,
        io: 0,
    };

    let mut probe = 0usize;
    while probe < M {
        let time = allocations[probe].start;
        let mut active = Allocation {
            cpu: 0,
            memory: 0,
            io: 0,
        };

        let mut candidate = 0usize;
        while candidate < M {
            let item = allocations[candidate];
            if item.start <= time && time < item.end {
                active = checked_add_allocation(active, item.allocation)
                    .ok_or(SymbolicRefusal::ArithmeticOverflow)?;
            }
            candidate += 1;
        }

        peak.cpu = core::cmp::max(peak.cpu, active.cpu);
        peak.memory = core::cmp::max(peak.memory, active.memory);
        peak.io = core::cmp::max(peak.io, active.io);
        probe += 1;
    }

    Ok(peak)
}

/// Loom-style pipeline-boundary choices for a load/body/store execution node.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PipelineKnobs {
    pub load_body_open: bool,
    pub body_store_open: bool,
}

/// Compose load/body/store costs using the HEG pipeline equations from
/// arXiv:2609.29219. This models overlap only; it does not select a candidate.
pub fn pipeline_latency(
    load: u64,
    body: u64,
    store: u64,
    knobs: PipelineKnobs,
) -> Result<u64, SymbolicRefusal> {
    match (knobs.load_body_open, knobs.body_store_open) {
        (false, false) => load
            .checked_add(body)
            .and_then(|value| value.checked_add(store))
            .ok_or(SymbolicRefusal::ArithmeticOverflow),
        (false, true) => {
            let load_body = load
                .checked_add(body)
                .ok_or(SymbolicRefusal::ArithmeticOverflow)?;
            Ok(core::cmp::max(load_body, store))
        }
        (true, false) => {
            let body_store = body
                .checked_add(store)
                .ok_or(SymbolicRefusal::ArithmeticOverflow)?;
            Ok(core::cmp::max(load, body_store))
        }
        (true, true) => Ok(core::cmp::max(load, core::cmp::max(body, store))),
    }
}
