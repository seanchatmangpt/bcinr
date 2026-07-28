//! Resource admission over canonical integer logical time.

use std::collections::BTreeMap;
use crate::logical_time::{LogicalTime, TimeConversionError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceMode { Exclusive, Shared }

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Resource {
    pub name: String,
    pub capacity: u32,
    pub mode: ResourceMode,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ResourceRefusal {
    Conflict { resource_id: String, overlap_interval: (f64, f64) },
    InvalidExternalTime { boundary: &'static str, reason: TimeConversionError },
    InvalidInterval { start: LogicalTime, end: LogicalTime },
    ZeroCapacity { resource_id: String },
    ResourceDefinitionMismatch { resource_id: String },
    LeaseIdExhausted,
    LeaseNotFound { resource_id: String, lease_id: u64 },
}

impl std::fmt::Display for ResourceRefusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Conflict { resource_id, overlap_interval } => write!(f, "resource conflict on {resource_id} at [{}, {})", overlap_interval.0, overlap_interval.1),
            Self::InvalidExternalTime { boundary, reason } => write!(f, "invalid {boundary} time: {reason}"),
            Self::InvalidInterval { start, end } => write!(f, "invalid interval [{start}, {end})"),
            Self::ZeroCapacity { resource_id } => write!(f, "resource {resource_id} has zero capacity"),
            Self::ResourceDefinitionMismatch { resource_id } => write!(f, "resource profile drift for {resource_id}"),
            Self::LeaseIdExhausted => write!(f, "lease identifier space exhausted"),
            Self::LeaseNotFound { resource_id, lease_id } => write!(f, "lease {lease_id} for {resource_id} is not active"),
        }
    }
}
impl std::error::Error for ResourceRefusal {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceLease {
    pub resource: Resource,
    start: LogicalTime,
    end: LogicalTime,
    lease_id: u64,
}

impl ResourceLease {
    #[must_use]
    pub fn interval(&self) -> (f64, f64) { (self.start.as_seconds_f64(), self.end.as_seconds_f64()) }
    #[must_use]
    pub const fn logical_interval(&self) -> (LogicalTime, LogicalTime) { (self.start, self.end) }
    #[must_use]
    pub const fn lease_id(&self) -> u64 { self.lease_id }
}

pub struct ResourceLedger {
    leases: BTreeMap<String, BTreeMap<u64, ResourceLease>>,
    next_id: u64,
}

impl ResourceLedger {
    #[must_use]
    pub fn new() -> Self { Self { leases: BTreeMap::new(), next_id: 0 } }

    pub fn request_lease(&mut self, resource: Resource, start: f64, end: f64) -> Result<ResourceLease, ResourceRefusal> {
        let start = LogicalTime::try_from_seconds_f64(start).map_err(|reason| ResourceRefusal::InvalidExternalTime { boundary: "start", reason })?;
        let end = LogicalTime::try_from_seconds_f64(end).map_err(|reason| ResourceRefusal::InvalidExternalTime { boundary: "end", reason })?;
        self.request_lease_at(resource, start, end)
    }

    pub fn request_lease_at(&mut self, resource: Resource, start: LogicalTime, end: LogicalTime) -> Result<ResourceLease, ResourceRefusal> {
        if start >= end { return Err(ResourceRefusal::InvalidInterval { start, end }); }
        if resource.capacity == 0 { return Err(ResourceRefusal::ZeroCapacity { resource_id: resource.name.clone() }); }

        if let Some(existing) = self.leases.get(&resource.name) {
            if existing.values().next().is_some_and(|lease| lease.resource != resource) {
                return Err(ResourceRefusal::ResourceDefinitionMismatch { resource_id: resource.name.clone() });
            }
            match resource.mode {
                ResourceMode::Exclusive => {
                    if let Some(lease) = existing.values().find(|lease| overlaps((lease.start, lease.end), (start, end))) {
                        return Err(conflict(&resource.name, intersection((lease.start, lease.end), (start, end))));
                    }
                }
                ResourceMode::Shared => {
                    let count = existing.values().filter(|lease| overlaps((lease.start, lease.end), (start, end))).count();
                    if count >= resource.capacity as usize { return Err(conflict(&resource.name, (start, end))); }
                }
            }
        }

        let id = self.next_id;
        self.next_id = self.next_id.checked_add(1).ok_or(ResourceRefusal::LeaseIdExhausted)?;
        let lease = ResourceLease { resource, start, end, lease_id: id };
        self.leases.entry(lease.resource.name.clone()).or_default().insert(id, lease.clone());
        Ok(lease)
    }

    /// Returns false for an unknown lease; release is never a silent no-op.
    pub fn release_lease(&mut self, lease: &ResourceLease) -> bool {
        self.leases.get_mut(&lease.resource.name).and_then(|set| set.remove(&lease.lease_id)).is_some()
    }

    pub fn renew_lease_at(&mut self, lease: &ResourceLease, start: LogicalTime, end: LogicalTime) -> Result<ResourceLease, ResourceRefusal> {
        if !self.release_lease(lease) {
            return Err(ResourceRefusal::LeaseNotFound { resource_id: lease.resource.name.clone(), lease_id: lease.lease_id });
        }
        match self.request_lease_at(lease.resource.clone(), start, end) {
            Ok(renewed) => Ok(renewed),
            Err(error) => {
                self.leases.entry(lease.resource.name.clone()).or_default().insert(lease.lease_id, lease.clone());
                Err(error)
            }
        }
    }

    #[must_use]
    pub fn lease_count(&self, name: &str) -> usize { self.leases.get(name).map_or(0, BTreeMap::len) }

    #[must_use]
    pub fn leases_for_resource(&self, name: &str) -> Vec<ResourceLease> {
        self.leases.get(name).map(|set| set.values().cloned().collect()).unwrap_or_default()
    }
}
impl Default for ResourceLedger { fn default() -> Self { Self::new() } }

fn overlaps(a: (LogicalTime, LogicalTime), b: (LogicalTime, LogicalTime)) -> bool { a.0 < b.1 && b.0 < a.1 }
fn intersection(a: (LogicalTime, LogicalTime), b: (LogicalTime, LogicalTime)) -> (LogicalTime, LogicalTime) { (a.0.max(b.0), a.1.min(b.1)) }
fn conflict(resource_id: &str, overlap: (LogicalTime, LogicalTime)) -> ResourceRefusal {
    ResourceRefusal::Conflict { resource_id: resource_id.into(), overlap_interval: (overlap.0.as_seconds_f64(), overlap.1.as_seconds_f64()) }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn cpu() -> Resource { Resource { name: "cpu".into(), capacity: 1, mode: ResourceMode::Exclusive } }

    #[test]
    fn overlap_refuses_without_mutation() {
        let mut ledger = ResourceLedger::new();
        ledger.request_lease(cpu(), 0.0, 10.0).unwrap();
        assert!(matches!(ledger.request_lease(cpu(), 5.0, 15.0), Err(ResourceRefusal::Conflict { overlap_interval: (5.0, 10.0), .. })));
        assert_eq!(ledger.lease_count("cpu"), 1);
    }

    #[test]
    fn invalid_external_time_refuses() {
        let mut ledger = ResourceLedger::new();
        assert!(matches!(ledger.request_lease(cpu(), f64::NAN, 1.0), Err(ResourceRefusal::InvalidExternalTime { .. })));
    }
}
