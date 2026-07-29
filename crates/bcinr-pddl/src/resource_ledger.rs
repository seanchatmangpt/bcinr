//! Interval-aware resource ledger for temporal planning.
//!
//! Tracks resource ownership across time intervals with exclusive/shared modes,
//! lease expiration, and renewal admission. Mirrors the `admit_proposal`
//! pattern from `bcinr_cmca::proposal` for resource conflict detection.
//!
//! # Invariants
//!
//! - All intervals are half-open: [t_start, t_end)
//! - Exclusive resources cannot be held by multiple leases concurrently
//! - Shared resources respect capacity bounds across time intervals
//! - Released leases are immediately freed and can be re-requested

use std::collections::BTreeMap;

/// Resource modes determine conflict semantics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceMode {
    /// Only one lease at a time; any overlap is a conflict.
    Exclusive,
    /// Multiple leases allowed up to `capacity` units concurrently.
    Shared,
}

/// A resource with identity, capacity, and access mode.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Resource {
    /// Unique identifier for this resource.
    pub name: String,
    /// Maximum concurrent capacity (only applies to Shared mode).
    pub capacity: u32,
    /// Access mode (Exclusive or Shared).
    pub mode: ResourceMode,
}

/// Typed refusal for resource lease admission.
#[derive(Debug, Clone, PartialEq)]
pub enum ResourceRefusal {
    /// A lease interval conflicts with an existing lease on this resource.
    Conflict {
        /// The resource that caused the conflict.
        resource_id: String,
        /// The interval [t_start, t_end) where the conflict occurs.
        overlap_interval: (f64, f64),
    },
}

impl std::fmt::Display for ResourceRefusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Conflict {
                resource_id,
                overlap_interval,
            } => write!(
                f,
                "resource conflict on {} at interval [{}, {})",
                resource_id, overlap_interval.0, overlap_interval.1
            ),
        }
    }
}

impl std::error::Error for ResourceRefusal {}

/// A granted lease for a resource over a time interval.
#[derive(Debug, Clone, PartialEq)]
pub struct ResourceLease {
    /// The resource this lease holds.
    pub resource: Resource,
    /// Start time of the interval (inclusive).
    pub start: f64,
    /// End time of the interval (exclusive).
    pub end: f64,
    /// Unique lease ID for tracking and release.
    lease_id: u64,
}

impl ResourceLease {
    /// Return the interval as [start, end).
    pub fn interval(&self) -> (f64, f64) {
        (self.start, self.end)
    }
}

/// Manages resource leases across intervals.
pub struct ResourceLedger {
    /// Leases keyed by resource name, then by lease_id.
    leases_by_resource: BTreeMap<String, BTreeMap<u64, ResourceLease>>,
    /// Counter for unique lease IDs.
    next_lease_id: u64,
}

impl ResourceLedger {
    /// Create a new, empty resource ledger.
    pub fn new() -> Self {
        Self {
            leases_by_resource: BTreeMap::new(),
            next_lease_id: 0,
        }
    }

    /// Attempt to admit a lease for the given resource over [start, end).
    ///
    /// Re-admission follows the `admit_proposal`
    /// pattern: every binding is re-verified before granting. If the interval
    /// conflicts with an existing lease:
    ///   - For **Exclusive** mode: any overlap is refused.
    ///   - For **Shared** mode: overlap is refused only if total capacity is exceeded.
    ///
    /// # Errors
    ///
    /// Returns [`ResourceRefusal::Conflict`] if the requested interval overlaps
    /// with an existing lease in a way that violates the resource's mode.
    pub fn request_lease(
        &mut self,
        resource: Resource,
        start: f64,
        end: f64,
    ) -> Result<ResourceLease, ResourceRefusal> {
        if start >= end {
            return Err(ResourceRefusal::Conflict {
                resource_id: resource.name.clone(),
                overlap_interval: (start, end),
            });
        }

        let existing_leases = self
            .leases_by_resource
            .entry(resource.name.clone())
            .or_default();

        match resource.mode {
            ResourceMode::Exclusive => {
                // Check if any existing lease overlaps with [start, end)
                for lease in existing_leases.values() {
                    if intervals_overlap((lease.start, lease.end), (start, end)) {
                        return Err(ResourceRefusal::Conflict {
                            resource_id: resource.name.clone(),
                            overlap_interval: overlap_region(
                                (lease.start, lease.end),
                                (start, end),
                            ),
                        });
                    }
                }
            }
            ResourceMode::Shared => {
                // Check if capacity is exceeded at any point in [start, end)
                let total_capacity = resource.capacity as usize;
                let overlapping_count = existing_leases
                    .values()
                    .filter(|lease| intervals_overlap((lease.start, lease.end), (start, end)))
                    .count();

                if overlapping_count >= total_capacity {
                    return Err(ResourceRefusal::Conflict {
                        resource_id: resource.name.clone(),
                        overlap_interval: (start, end),
                    });
                }
            }
        }

        // Admission successful: create lease and track it
        let lease_id = self.next_lease_id;
        self.next_lease_id += 1;

        let lease = ResourceLease {
            resource,
            start,
            end,
            lease_id,
        };

        existing_leases.insert(lease_id, lease.clone());
        Ok(lease)
    }

    /// Release a previously-admitted lease, freeing the resource interval.
    ///
    /// After release, the interval becomes available for future requests.
    /// Releasing a lease that is not currently held is a no-op.
    pub fn release_lease(&mut self, lease: &ResourceLease) {
        if let Some(resource_leases) = self.leases_by_resource.get_mut(&lease.resource.name) {
            resource_leases.remove(&lease.lease_id);
        }
    }

    /// Return the number of active leases on a resource.
    pub fn lease_count(&self, resource_name: &str) -> usize {
        self.leases_by_resource
            .get(resource_name)
            .map(|m| m.len())
            .unwrap_or(0)
    }

    /// Return all active leases for a resource (in arbitrary order).
    pub fn leases_for_resource(&self, resource_name: &str) -> Vec<ResourceLease> {
        self.leases_by_resource
            .get(resource_name)
            .map(|m| m.values().cloned().collect())
            .unwrap_or_default()
    }
}

impl Default for ResourceLedger {
    fn default() -> Self {
        Self::new()
    }
}

/// Check if two intervals [a_start, a_end) and [b_start, b_end) overlap.
fn intervals_overlap(a: (f64, f64), b: (f64, f64)) -> bool {
    a.0 < b.1 && b.0 < a.1
}

/// Return the overlapping region of two intervals, or the request interval if no overlap.
fn overlap_region(a: (f64, f64), b: (f64, f64)) -> (f64, f64) {
    if !intervals_overlap(a, b) {
        return b;
    }
    (a.0.max(b.0), a.1.min(b.1))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resource_mode_variants() {
        assert_ne!(ResourceMode::Exclusive, ResourceMode::Shared);
    }

    #[test]
    fn new_ledger_is_empty() {
        let ledger = ResourceLedger::new();
        assert_eq!(ledger.lease_count("cpu"), 0);
    }

    #[test]
    fn exclusive_resource_single_lease() {
        let mut ledger = ResourceLedger::new();
        let cpu = Resource {
            name: "cpu".to_string(),
            capacity: 1,
            mode: ResourceMode::Exclusive,
        };

        let lease1 = ledger.request_lease(cpu.clone(), 0.0, 10.0);
        assert!(lease1.is_ok());
        assert_eq!(ledger.lease_count("cpu"), 1);
    }

    #[test]
    fn exclusive_resource_non_overlapping_ok() {
        let mut ledger = ResourceLedger::new();
        let cpu = Resource {
            name: "cpu".to_string(),
            capacity: 1,
            mode: ResourceMode::Exclusive,
        };

        let lease1 = ledger.request_lease(cpu.clone(), 0.0, 10.0);
        assert!(lease1.is_ok());

        let lease2 = ledger.request_lease(cpu.clone(), 10.0, 20.0);
        assert!(lease2.is_ok());

        assert_eq!(ledger.lease_count("cpu"), 2);
    }

    #[test]
    fn exclusive_resource_overlapping_refused() {
        let mut ledger = ResourceLedger::new();
        let cpu = Resource {
            name: "cpu".to_string(),
            capacity: 1,
            mode: ResourceMode::Exclusive,
        };

        let lease1 = ledger.request_lease(cpu.clone(), 0.0, 10.0);
        assert!(lease1.is_ok());

        let lease2 = ledger.request_lease(cpu.clone(), 5.0, 15.0);
        assert!(matches!(
            lease2,
            Err(ResourceRefusal::Conflict {
                resource_id,
                overlap_interval
            }) if resource_id == "cpu" && overlap_interval == (5.0, 10.0)
        ));

        assert_eq!(ledger.lease_count("cpu"), 1);
    }

    #[test]
    fn release_frees_interval() {
        let mut ledger = ResourceLedger::new();
        let cpu = Resource {
            name: "cpu".to_string(),
            capacity: 1,
            mode: ResourceMode::Exclusive,
        };

        let lease1 = ledger.request_lease(cpu.clone(), 0.0, 10.0).unwrap();
        assert_eq!(ledger.lease_count("cpu"), 1);

        ledger.release_lease(&lease1);
        assert_eq!(ledger.lease_count("cpu"), 0);

        // Now the same interval can be requested again
        let lease2 = ledger.request_lease(cpu, 0.0, 10.0);
        assert!(lease2.is_ok());
        assert_eq!(ledger.lease_count("cpu"), 1);
    }

    #[test]
    fn shared_resource_respects_capacity() {
        let mut ledger = ResourceLedger::new();
        let workers = Resource {
            name: "workers".to_string(),
            capacity: 2,
            mode: ResourceMode::Shared,
        };

        let lease1 = ledger.request_lease(workers.clone(), 0.0, 10.0);
        assert!(lease1.is_ok());

        let lease2 = ledger.request_lease(workers.clone(), 5.0, 15.0);
        assert!(lease2.is_ok());

        // Third lease on overlapping interval should be refused
        let lease3 = ledger.request_lease(workers, 7.0, 12.0);
        assert!(matches!(lease3, Err(ResourceRefusal::Conflict { .. })));

        assert_eq!(ledger.lease_count("workers"), 2);
    }
}
