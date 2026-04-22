//! Aggregate Patterns: Higher-order constructs built from branchless primitives.

pub mod bloom_scan;
pub mod swar_petri;
pub mod deterministic_mpmc;
pub mod autonomic_arena;
pub mod bit_transcoder;
pub mod policy_dfa;
pub mod integrity_receipt;
pub mod wait_free_multicast;
pub mod wcet_fiber;
pub mod register_sql;

// Advanced Abstractions
pub mod matrix_lru;
pub mod chacha_sponge;
pub mod swar_quotient;
pub mod bitonic_pq;
pub mod hazard_shield;
pub mod radix_trie;
pub mod consensus_bft;
pub mod time_wheel;

pub use bloom_scan::BloomScanPipeline;
pub use swar_petri::PriorityPetriEngine;
pub use deterministic_mpmc::LockFreeMpmcRing;
pub use autonomic_arena::AutonomicExhaustionArena;
pub use bit_transcoder::BitTranscoder;
pub use policy_dfa::ConstantShapePolicyDfa;
pub use integrity_receipt::DeterministicSubstrateReceipt;
pub use wait_free_multicast::BoundedSpscMulticast;
pub use wcet_fiber::WcetFiber;
pub use register_sql::RegisterEngine;

// Advanced Exports
pub use matrix_lru::MatrixLru;
pub use chacha_sponge::ChaChaSponge;
pub use swar_quotient::SwarQuotientFilter;
pub use bitonic_pq::BitonicPriorityQueue8;
pub use hazard_shield::HazardShield;
pub use radix_trie::RadixTrieNode;
pub use consensus_bft::FixedConsensus;
pub use time_wheel::TimeWheel;

#[cfg(test)]
mod tests;
