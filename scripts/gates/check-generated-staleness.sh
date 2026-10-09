#!/usr/bin/env bash
# Generated-artifact staleness gate (exhaustive.yml's "Generated-artifact
# standing" step).
#
# Regenerates every in-repo generated artifact from its admitted source and
# fails if the committed copy differs. The generation pipeline is
# generator.py -> rustfmt, so the check applies the same normalization
# before diffing.
#
# In scope (regenerable inside this repo):
#   crates/bcinr-cmca/src/generated/consequence_mass/{case_studies,generalization}.rs
#     from crates/bcinr-cmca/ontology/{cmca-rdf,generalization}.ttl
# Out of scope (provenance is the external ggen tool, not generator.py --
# cannot be regenerated from inside this repo):
#   stability_profile.rs, generated_profile.rs
set -euo pipefail
cd "$(dirname "$0")/../../crates/bcinr-cmca"

fail=0
check() {
    local ttl="$1" out="$2" tmp
    tmp="$(mktemp -t staleness.XXXXXX.rs)"
    python3 generator.py "$ttl" "$tmp" >/dev/null
    rustfmt "$tmp" >/dev/null 2>&1 || true
    if ! diff -q "$tmp" "$out" >/dev/null; then
        echo "STALE: $out does not match regeneration from $ttl"
        diff "$tmp" "$out" | head -10
        fail=1
    else
        echo "OK: $out in sync with $ttl"
    fi
    rm -f "$tmp"
}

check ontology/cmca-rdf.ttl src/generated/consequence_mass/case_studies.rs
check ontology/generalization.ttl src/generated/consequence_mass/generalization.rs
exit $fail
