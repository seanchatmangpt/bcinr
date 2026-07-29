//! causal_receipt — OCEL causal frame and rolling BLAKE3 receipt chain.
//!
//! The chain follows the same `causal_mix` discipline as `unibit-causality`:
//! each frame's BLAKE3 digest is computed over `prior_hash || frame_bytes`
//! (all fields serialised little-endian), advancing the chain by one step.

use crate::receipt::denial::DenialPolarity;

// ── PackedObjRef ─────────────────────────────────────────────────────────────

/// A packed object reference encoding type index (high 8 bits) and object id
/// (low 24 bits) in a single `u32`.
#[repr(transparent)]
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct PackedObjRef(pub u32);

impl PackedObjRef {
    /// Pack `type_idx` and `object_id` into one word.
    ///
    /// `object_id` is truncated to 24 bits; the high 8 bits of `object_id` are
    /// silently dropped.
    #[inline]
    pub fn new(type_idx: u8, object_id: u32) -> Self {
        Self(((type_idx as u32) << 24) | (object_id & 0x00FF_FFFF))
    }

    /// The type index (high 8 bits).
    #[inline]
    pub fn type_idx(self) -> u8 {
        (self.0 >> 24) as u8
    }

    /// The object id (low 24 bits).
    #[inline]
    pub fn object_id(self) -> u32 {
        self.0 & 0x00FF_FFFF
    }
}

// ── OcelCausalFrame ──────────────────────────────────────────────────────────

/// One OCEL causal frame: a cache-line-sized record of a single manufacturing
/// step, its denial verdict, its object set, and the rolling hash of its
/// causal predecessor.
///
/// Size: 128 bytes (2 × 64-byte cache lines), aligned to 64 bytes.
#[derive(Clone)]
#[repr(C, align(64))]
pub struct OcelCausalFrame {
    /// Monotonically increasing step identity within a run.
    pub instruction_id: u64,
    /// Scatter of active denial lanes (from [`DenialPolarity::to_fired_mask`]).
    pub fired_mask: u64,
    /// Denial polarity at the time this step was manufactured.
    pub denial: DenialPolarity,
    /// Up to 8 packed object references participating in this step.
    pub obj_refs: [PackedObjRef; 8],
    /// Wall-clock timestamp in nanoseconds.
    pub ts_ns: u64,
    /// Index into the [`crate::receipt::intern::ActivityTable`] for this step's activity.
    pub activity_idx: u16,
    /// Classifier byte for the POWL node kind (XOR, SEQ, LOOP, etc.).
    pub node_kind: u8,
    /// Internal padding to maintain 128-byte alignment.
    pub pad: [u8; 5],
    /// BLAKE3 hash of the preceding frame (or genesis zeros for the first frame).
    pub prior_hash: [u8; 32],
}

// Compile-time size assertion: the struct must be exactly 128 bytes.
const _: () = {
    assert!(
        core::mem::size_of::<OcelCausalFrame>() == 128,
        "OcelCausalFrame must be exactly 128 bytes"
    );
};

impl OcelCausalFrame {
    /// Serialise this frame into a fixed-size byte buffer for hashing.
    ///
    /// Layout (all integers little-endian):
    /// ```text
    /// [  0.. 8]  instruction_id  (u64 LE)
    /// [  8..16]  fired_mask      (u64 LE)
    /// [ 16..24]  denial.0        (u64 LE)
    /// [ 24..56]  obj_refs        (8 × u32 LE)
    /// [ 56..64]  ts_ns           (u64 LE)
    /// [ 64..66]  activity_idx    (u16 LE)
    /// [ 66..67]  node_kind       (u8)
    /// [ 67..99]  prior_hash      (32 bytes verbatim)
    /// ```
    /// Total: 99 bytes.
    fn to_hash_bytes(&self) -> [u8; 99] {
        let mut buf = [0u8; 99];
        let mut pos = 0;

        // instruction_id
        for i in 0..8 {
            buf[pos + i] = ((self.instruction_id >> (i * 8)) & 0xFF) as u8;
        }
        pos += 8;

        // fired_mask
        for i in 0..8 {
            buf[pos + i] = ((self.fired_mask >> (i * 8)) & 0xFF) as u8;
        }
        pos += 8;

        // denial.0
        for i in 0..8 {
            buf[pos + i] = ((self.denial.0 >> (i * 8)) & 0xFF) as u8;
        }
        pos += 8;

        // obj_refs (8 × u32 LE)
        for r in &self.obj_refs {
            let v = r.0;
            for i in 0..4 {
                buf[pos + i] = ((v >> (i * 8)) & 0xFF) as u8;
            }
            pos += 4;
        }

        // ts_ns
        for i in 0..8 {
            buf[pos + i] = ((self.ts_ns >> (i * 8)) & 0xFF) as u8;
        }
        pos += 8;

        // activity_idx (u16 LE)
        buf[pos] = (self.activity_idx & 0xFF) as u8;
        buf[pos + 1] = ((self.activity_idx >> 8) & 0xFF) as u8;
        pos += 2;

        // node_kind
        buf[pos] = self.node_kind;
        pos += 1;

        // prior_hash
        buf[pos..pos + 32].copy_from_slice(&self.prior_hash);

        buf
    }
}

// ── OcelCausalReceipt ────────────────────────────────────────────────────────

/// Rolling BLAKE3 receipt for an ordered sequence of [`OcelCausalFrame`]s.
///
/// The chain invariant mirrors `unibit-causality`:
/// ```text
/// chain_hash(t+1) = BLAKE3(chain_hash(t) || frame_bytes(t+1))
/// ```
/// The genesis hash is BLAKE3 of 32 zero bytes.
pub struct OcelCausalReceipt {
    /// Current rolling hash (advances with each [`OcelCausalReceipt::chain`] call).
    pub chain_hash: [u8; 32],
    /// Number of frames chained so far.
    pub frame_count: u64,
    /// Opaque run identifier supplied at genesis.
    pub run_id: [u8; 32],
    /// Replay pointer: index of the last frame that can serve as a replay root.
    pub replay_ptr: u64,
}

impl OcelCausalReceipt {
    /// Create a genesis receipt for the given `run_id`.
    ///
    /// The initial `chain_hash` is BLAKE3 of 32 zero bytes, matching the
    /// `unibit-causality` genesis convention.
    pub fn genesis(run_id: [u8; 32]) -> Self {
        let chain_hash: [u8; 32] = *blake3::hash(&[0u8; 32]).as_bytes();
        Self {
            chain_hash,
            frame_count: 0,
            run_id,
            replay_ptr: 0,
        }
    }

    /// Advance the chain by one frame.
    ///
    /// Computes `BLAKE3(chain_hash || frame.to_hash_bytes())` and stores the
    /// result as the new `chain_hash`.  Also records `frame_count` and advances
    /// `replay_ptr` to the new frame index.
    pub fn chain(&mut self, frame: &OcelCausalFrame) {
        let frame_bytes = frame.to_hash_bytes();
        // Streaming update: feed prior_hash then frame_bytes in one Hasher pass.
        // Avoids a 131-byte stack copy and a second Hasher construction vs one-shot.
        let mut h = blake3::Hasher::new();
        h.update(&self.chain_hash);
        h.update(&frame_bytes);
        self.chain_hash = *h.finalize().as_bytes();
        self.frame_count += 1;
        self.replay_ptr = self.frame_count - 1;
    }

    /// Produce the canonical hash string `"blake3:<64 hex chars>"`.
    ///
    /// The returned array is exactly 71 bytes: `"blake3:"` (7) + 64 hex digits.
    pub fn canonical_hash(&self) -> [u8; 71] {
        let mut out = [0u8; 71];
        out[..7].copy_from_slice(b"blake3:");
        let hex = hex_encode_32(&self.chain_hash);
        out[7..].copy_from_slice(&hex);
        out
    }
}

/// Encode 32 bytes as 64 lowercase hex ASCII digits.
fn hex_encode_32(bytes: &[u8; 32]) -> [u8; 64] {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = [0u8; 64];
    for (i, &b) in bytes.iter().enumerate() {
        out[i * 2] = HEX[(b >> 4) as usize];
        out[i * 2 + 1] = HEX[(b & 0xF) as usize];
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn genesis_frame(instruction_id: u64, denial: DenialPolarity) -> OcelCausalFrame {
        OcelCausalFrame {
            instruction_id,
            fired_mask: denial.to_fired_mask(),
            denial,
            obj_refs: [PackedObjRef::default(); 8],
            ts_ns: 1_000_000,
            activity_idx: 0,
            node_kind: 0,
            pad: [0u8; 5],
            prior_hash: [0u8; 32],
        }
    }

    #[test]
    fn frame_size_is_128() {
        assert_eq!(core::mem::size_of::<OcelCausalFrame>(), 128);
    }

    #[test]
    fn packed_obj_ref_roundtrip() {
        let r = PackedObjRef::new(0xAB, 0x00C0FFEE);
        assert_eq!(r.type_idx(), 0xAB);
        // object_id is 24-bit; 0x00C0FFEE & 0xFFFFFF = 0xC0FFEE
        assert_eq!(r.object_id(), 0x00C0FFEE & 0x00FF_FFFF);
    }

    #[test]
    fn genesis_receipt_deterministic() {
        let run_id = [0xABu8; 32];
        let r1 = OcelCausalReceipt::genesis(run_id);
        let r2 = OcelCausalReceipt::genesis(run_id);
        assert_eq!(r1.chain_hash, r2.chain_hash);
        assert_eq!(r1.frame_count, 0);
    }

    #[test]
    fn chain_advances_hash() {
        let run_id = [0u8; 32];
        let mut receipt = OcelCausalReceipt::genesis(run_id);
        let before = receipt.chain_hash;
        let frame = genesis_frame(1, DenialPolarity::ADMITTED);
        receipt.chain(&frame);
        assert_ne!(receipt.chain_hash, before, "chain must advance the hash");
        assert_eq!(receipt.frame_count, 1);
    }

    #[test]
    fn chain_different_denial_gives_different_hash() {
        let run_id = [0u8; 32];
        let mut r_admitted = OcelCausalReceipt::genesis(run_id);
        let mut r_denied = OcelCausalReceipt::genesis(run_id);

        let f_admitted = genesis_frame(1, DenialPolarity::ADMITTED);
        let f_denied = genesis_frame(1, DenialPolarity::PRECONDITION_FAILED);

        r_admitted.chain(&f_admitted);
        r_denied.chain(&f_denied);

        assert_ne!(
            r_admitted.chain_hash, r_denied.chain_hash,
            "different denial polarities must produce different hashes"
        );
    }

    #[test]
    fn canonical_hash_prefix_and_length() {
        let run_id = [0u8; 32];
        let receipt = OcelCausalReceipt::genesis(run_id);
        let canonical = receipt.canonical_hash();
        assert_eq!(&canonical[..7], b"blake3:");
        assert_eq!(canonical.len(), 71);
        // All hex digits must be lowercase ASCII hex.
        for &b in &canonical[7..] {
            assert!(b.is_ascii_hexdigit(), "expected hex digit, got {b:#04x}");
        }
    }

    #[test]
    fn chain_deterministic() {
        let run_id = [42u8; 32];
        let frame = genesis_frame(99, DenialPolarity::SLA_BREACH);

        let mut r1 = OcelCausalReceipt::genesis(run_id);
        let mut r2 = OcelCausalReceipt::genesis(run_id);
        r1.chain(&frame);
        r2.chain(&frame);
        assert_eq!(
            r1.chain_hash, r2.chain_hash,
            "chaining must be deterministic"
        );
    }

    #[test]
    fn replay_ptr_tracks_last_frame() {
        let run_id = [0u8; 32];
        let mut receipt = OcelCausalReceipt::genesis(run_id);
        for i in 0..5u64 {
            receipt.chain(&genesis_frame(i, DenialPolarity::ADMITTED));
            assert_eq!(receipt.replay_ptr, i);
        }
    }
}
