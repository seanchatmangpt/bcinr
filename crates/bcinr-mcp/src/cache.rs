//! MCP+ admission-key cache — the buildable 80/20 slice of the "lawful
//! capability engine" PRD.
//!
//! The PRD's full cache key has 8 provenance fields (capability id/version,
//! ontology digest, policy digest, authority digest, O* digest, replay mode,
//! environment digest). Only one of those has a real referent in this
//! codebase today: the input itself. This cache keys on `(tool name,
//! BLAKE3(canonical input))` — the honest subset — rather than fabricating
//! placeholder values for ontology/policy/authority systems that don't
//! exist here. When those systems exist, extend the key; until then, a
//! content-addressed cache on the input is the real, load-bearing part of
//! the PRD's idea: identical input to a pure tool call is (and should be
//! treated as) the same admitted consequence, not recomputed from scratch.
//!
//! Only used for tools that are pure functions of their input with no
//! external mutable state — `manufacture_world` and `pddl_plan` today. A
//! side-effecting or non-deterministic tool must never be wrapped in this
//! cache.

use moka::future::Cache;
use std::time::Duration;

/// Shared cache: key is `"{tool}:{blake3_hex(input)}"`, value is the tool's
/// existing JSON-string response (unchanged response shape — no new schema).
///
/// TTL is bounded (not unbounded) even though these tools are pure: there is
/// no policy/ontology-versioning system in this codebase yet to invalidate
/// on, so a bounded TTL is the honest stand-in for "this consequence is
/// still admissible" rather than claiming eternal validity.
#[derive(Clone)]
pub struct CapabilityCache {
    inner: Cache<String, String>,
}

impl CapabilityCache {
    pub fn new() -> Self {
        Self {
            inner: Cache::builder()
                .max_capacity(10_000)
                .time_to_live(Duration::from_secs(5 * 60))
                .build(),
        }
    }

    /// Build the cache key for `tool` given its already-serialized canonical
    /// input bytes (e.g. `serde_json::to_vec(&input)`).
    pub fn key(tool: &str, canonical_input: &[u8]) -> String {
        format!("{tool}:{}", blake3::hash(canonical_input).to_hex())
    }

    pub async fn get(&self, key: &str) -> Option<String> {
        self.inner.get(key).await
    }

    pub async fn insert(&self, key: String, value: String) {
        self.inner.insert(key, value).await;
    }
}

impl Default for CapabilityCache {
    fn default() -> Self {
        Self::new()
    }
}

/// The "law object" cache key shape: `tool`/`input_hash` are always present
/// (the only two fields with a real referent in this codebase today);
/// everything else is `Option` because no policy/authority/environment/
/// replay-mode system exists yet to populate them.
///
/// **The load-bearing property**: a field only affects the resulting key
/// string when it is `Some`. A `None` field is *absent* from the key, not
/// present-as-empty — so two calls that both leave a field unset produce the
/// same key regardless of what that field "could have been" (unset fields
/// do not enforce law), while two calls that both set a field to different
/// values produce different keys (set fields do enforce law). This is
/// proven directly by `tests/cache_law_field_matrix.rs`, not just asserted
/// in this comment — see that test for the exact claim being checked.
///
/// Not yet wired into `manufacture_world`/`pddl_plan` (those still use the
/// plain `tool`/`input_hash` two-field key via `CapabilityCache::key`) —
/// this struct exists to prove the shape is honest before promoting it to
/// production use, per this session's "prove it, then use it" discipline.
#[derive(Debug, Clone, Copy)]
pub struct CapabilityCacheKey<'a> {
    pub tool: &'a str,
    pub input_hash: &'a str,
    pub capability_version: Option<&'a str>,
    pub policy_digest: Option<&'a str>,
    pub authority_digest: Option<&'a str>,
    pub environment_digest: Option<&'a str>,
    pub replay_mode: Option<&'a str>,
}

impl<'a> CapabilityCacheKey<'a> {
    /// Render this key to its composite cache-key string. Field order is
    /// fixed and stable; `None` fields contribute nothing (not even a
    /// placeholder marker) to the output.
    pub fn to_key_string(&self) -> String {
        let mut s = format!("{}:{}", self.tool, self.input_hash);
        if let Some(v) = self.capability_version {
            s.push_str(&format!(":cv={v}"));
        }
        if let Some(v) = self.policy_digest {
            s.push_str(&format!(":pd={v}"));
        }
        if let Some(v) = self.authority_digest {
            s.push_str(&format!(":ad={v}"));
        }
        if let Some(v) = self.environment_digest {
            s.push_str(&format!(":ed={v}"));
        }
        if let Some(v) = self.replay_mode {
            s.push_str(&format!(":rm={v}"));
        }
        s
    }
}

#[cfg(test)]
mod law_field_matrix_tests {
    //! Proves the exact claim `CapabilityCacheKey`'s doc comment makes:
    //! `None` fields do not enforce law (identical keys regardless of what
    //! an unset field "could have been"), and `Some` fields do (different
    //! values produce different keys). This is the cache-key law-field
    //! matrix benchmark from `docs/`'s MCP+ cache proof suite.
    use super::CapabilityCacheKey;

    fn base<'a>() -> CapabilityCacheKey<'a> {
        CapabilityCacheKey {
            tool: "manufacture_world",
            input_hash: "deadbeef",
            capability_version: None,
            policy_digest: None,
            authority_digest: None,
            environment_digest: None,
            replay_mode: None,
        }
    }

    #[test]
    fn none_fields_do_not_enforce_law_identical_keys_when_all_unset() {
        let a = base();
        let b = base();
        assert_eq!(a.to_key_string(), b.to_key_string());
    }

    #[test]
    fn tool_or_input_hash_change_always_changes_key() {
        let mut other_tool = base();
        other_tool.tool = "pddl_plan";
        assert_ne!(base().to_key_string(), other_tool.to_key_string());

        let mut other_hash = base();
        other_hash.input_hash = "cafebabe";
        assert_ne!(base().to_key_string(), other_hash.to_key_string());
    }

    /// For every optional law field, setting it changes the key (law is
    /// enforced when present), and two different `Some` values for the
    /// same field produce two different keys (the field actually
    /// discriminates, not just "present vs absent").
    macro_rules! optional_field_enforces_law {
        ($test_name:ident, $field:ident) => {
            #[test]
            fn $test_name() {
                let unset = base();
                let mut set_a = base();
                set_a.$field = Some("v1");
                let mut set_b = base();
                set_b.$field = Some("v2");

                assert_ne!(
                    unset.to_key_string(),
                    set_a.to_key_string(),
                    "setting {} must change the key relative to it being unset",
                    stringify!($field)
                );
                assert_ne!(
                    set_a.to_key_string(),
                    set_b.to_key_string(),
                    "two different Some values for {} must produce different keys",
                    stringify!($field)
                );
            }
        };
    }

    optional_field_enforces_law!(capability_version_enforces_law, capability_version);
    optional_field_enforces_law!(policy_digest_enforces_law, policy_digest);
    optional_field_enforces_law!(authority_digest_enforces_law, authority_digest);
    optional_field_enforces_law!(environment_digest_enforces_law, environment_digest);
    optional_field_enforces_law!(replay_mode_enforces_law, replay_mode);

    /// Cross-check: leaving every optional field `None` on both sides,
    /// regardless of how many "law dimensions" exist, still yields the same
    /// key as the plain two-field case — unset fields never accumulate into
    /// an implicit, undocumented enforcement.
    #[test]
    fn all_fields_unset_matches_plain_two_field_key_semantics() {
        let full_key_all_none = base().to_key_string();
        let plain_key = format!("{}:{}", "manufacture_world", "deadbeef");
        assert_eq!(full_key_all_none, plain_key);
    }
}
