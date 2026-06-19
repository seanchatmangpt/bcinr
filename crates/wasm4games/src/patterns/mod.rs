//! Generated branchless pattern kernels.
//!
//! GENERATED SURFACE: the modules and the [`PATTERN_REGISTRY`] in this file are produced by
//! `ggen sync` from the input surface in `ggen/` and committed so the offline build never
//! needs ggen. Do not hand-edit generated kernels; edit `ggen/schema/patterns.ttl` and
//! regenerate. See the "GGEN-ONLY USER SURFACE" covenant in the docs.

use crate::ir::PatternSpec;

/// The catalog of all generated patterns as data. Iterated by [`crate::verify`].
///
/// Empty until kernels are generated; each generated kernel adds one [`PatternSpec`].
pub static PATTERN_REGISTRY: &[PatternSpec] = &[];
