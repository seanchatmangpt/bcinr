//! Production authority fence for the CMCA kernel.
//!
//! This module is intentionally small. It does not duplicate allocation,
//! certification, or verification algorithms. It makes the production role of
//! `bcinr-cmca` executable: BCINR may observe, analyze, allocate, certify, and
//! verify; it may not select an enterprise action, construct an external
//! artifact, or actuate a consequence.
//!
//! The mathematical implementation remains in [`crate::allocator`],
//! [`crate::observatory`], [`crate::stability_theorem`], and (with `alloc`)
//! [`crate::cascade`]. Consumers that need SELECT/CONSTRUCT/DO must cross a
//! separately admitted orchestration/authority boundary.

use crate::generated::consequence_mass::case_studies::{K, N, Q};

/// Production capabilities that can be requested at the CMCA boundary.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ProductionCapability {
    /// Observe bounded telemetry or state used by CMCA.
    Observe,
    /// Analyze/calibrate an observation without changing external state.
    Analyze,
    /// Compute consequence-mass allocation.
    Allocate,
    /// Certify a bounded CMCA result.
    Certify,
    /// Verify a CMCA result or invariant.
    Verify,
    /// Choose an enterprise/runtime action. Owned outside BCINR.
    Select,
    /// Construct an externally consequential artifact. Owned outside BCINR.
    Construct,
    /// Actuate an external consequence. Owned exclusively by the authority broker.
    Actuate,
}

/// Capabilities admitted to execute inside the BCINR production kernel.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum KernelCapability {
    /// Observation is non-actuating.
    Observe,
    /// Analysis is non-actuating.
    Analyze,
    /// CMCA consequence allocation.
    Allocate,
    /// CMCA certification.
    Certify,
    /// Independent verification.
    Verify,
}

/// Typed refusal when a caller attempts to widen BCINR's authority ceiling.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ProductionBoundaryRefusal {
    /// SELECT belongs to the orchestration/selection layer, not BCINR.
    SelectAuthorityExternal,
    /// CONSTRUCT belongs to an admitted constructor, not BCINR.
    ConstructAuthorityExternal,
    /// DO belongs to the sole consequential broker, never BCINR.
    ActuationAuthorityExternal,
}

/// Fixed generated CMCA shape used by the allocation-free hot path.
pub const FIXED_OBJECT_COUNT: usize = N;
/// Fixed number of generated CMCA measures.
pub const FIXED_MEASURE_COUNT: usize = K;
/// Fixed number of generated CMCA lenses.
pub const FIXED_LENS_COUNT: usize = Q;

/// Admit only capabilities that belong to the deterministic CMCA kernel.
///
/// This is an authority classification fence, not an execution API. Successful
/// admission means only that the requested capability belongs to BCINR's
/// production role. It does not manufacture a certificate, selection, permit,
/// operation, or actuation receipt.
pub const fn admit_kernel_capability(
    requested: ProductionCapability,
) -> Result<KernelCapability, ProductionBoundaryRefusal> {
    match requested {
        ProductionCapability::Observe => Ok(KernelCapability::Observe),
        ProductionCapability::Analyze => Ok(KernelCapability::Analyze),
        ProductionCapability::Allocate => Ok(KernelCapability::Allocate),
        ProductionCapability::Certify => Ok(KernelCapability::Certify),
        ProductionCapability::Verify => Ok(KernelCapability::Verify),
        ProductionCapability::Select => Err(ProductionBoundaryRefusal::SelectAuthorityExternal),
        ProductionCapability::Construct => {
            Err(ProductionBoundaryRefusal::ConstructAuthorityExternal)
        }
        ProductionCapability::Actuate => {
            Err(ProductionBoundaryRefusal::ActuationAuthorityExternal)
        }
    }
}

/// Production descriptor suitable for receipts, diagnostics, and cross-repo
/// compatibility tests without importing any execution authority.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct ProductionKernelDescriptor {
    /// Stable schema identifier for this boundary.
    pub schema: &'static str,
    /// Repository role in the production topology.
    pub role: &'static str,
    /// Whether this kernel owns SELECT authority.
    pub select_authority: bool,
    /// Whether this kernel owns CONSTRUCT authority.
    pub construct_authority: bool,
    /// Whether this kernel owns consequential DO authority.
    pub do_authority: bool,
    /// Generated fixed-width object count.
    pub objects: usize,
    /// Generated measure count.
    pub measures: usize,
    /// Generated lens count.
    pub lenses: usize,
}

/// Canonical production-role descriptor for `bcinr-cmca`.
pub const PRODUCTION_KERNEL: ProductionKernelDescriptor = ProductionKernelDescriptor {
    schema: "chatman.ppcx.kernel-boundary/v1",
    role: "cmca-kernel",
    select_authority: false,
    construct_authority: false,
    do_authority: false,
    objects: FIXED_OBJECT_COUNT,
    measures: FIXED_MEASURE_COUNT,
    lenses: FIXED_LENS_COUNT,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn production_kernel_admits_only_non_actuating_cmca_capabilities() {
        assert_eq!(
            admit_kernel_capability(ProductionCapability::Observe),
            Ok(KernelCapability::Observe)
        );
        assert_eq!(
            admit_kernel_capability(ProductionCapability::Analyze),
            Ok(KernelCapability::Analyze)
        );
        assert_eq!(
            admit_kernel_capability(ProductionCapability::Allocate),
            Ok(KernelCapability::Allocate)
        );
        assert_eq!(
            admit_kernel_capability(ProductionCapability::Certify),
            Ok(KernelCapability::Certify)
        );
        assert_eq!(
            admit_kernel_capability(ProductionCapability::Verify),
            Ok(KernelCapability::Verify)
        );
    }

    #[test]
    fn production_kernel_refuses_select_construct_and_do() {
        assert_eq!(
            admit_kernel_capability(ProductionCapability::Select),
            Err(ProductionBoundaryRefusal::SelectAuthorityExternal)
        );
        assert_eq!(
            admit_kernel_capability(ProductionCapability::Construct),
            Err(ProductionBoundaryRefusal::ConstructAuthorityExternal)
        );
        assert_eq!(
            admit_kernel_capability(ProductionCapability::Actuate),
            Err(ProductionBoundaryRefusal::ActuationAuthorityExternal)
        );
    }

    #[test]
    fn descriptor_carries_zero_external_authority() {
        assert_eq!(PRODUCTION_KERNEL.role, "cmca-kernel");
        assert!(!PRODUCTION_KERNEL.select_authority);
        assert!(!PRODUCTION_KERNEL.construct_authority);
        assert!(!PRODUCTION_KERNEL.do_authority);
        assert_eq!(PRODUCTION_KERNEL.objects, N);
        assert_eq!(PRODUCTION_KERNEL.measures, K);
        assert_eq!(PRODUCTION_KERNEL.lenses, Q);
    }
}
