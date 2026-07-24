#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

printf '\n==> Source formatting\n'
cargo fmt --all -- --check

printf '\n==> POWL v2 compiler and scheduler\n'
cargo test -p bcinr-powl --lib powl2
cargo test -p bcinr-powl --lib scheduler_v2

printf '\n==> POWL v2 receipt and replay\n'
cargo test -p bcinr-powl-receipt --lib execution_v2

printf '\n==> PDDL parser and exact classical semantics\n'
cargo test -p bcinr-pddl --features mfw-planner --test canonical_ipc
cargo test -p bcinr-pddl --features mfw-planner --lib ground_v2
cargo test -p bcinr-pddl --features mfw-planner --lib semantic_features
cargo test -p bcinr-pddl --features mfw-planner --lib production_capability

printf '\n==> PDDL to POWL execution rails\n'
cargo test -p bcinr-pddl --features mfw-planner --lib cognitive
cargo test -p bcinr-pddl --features mfw-planner --lib downstream
cargo test -p bcinr-pddl --features mfw-planner --lib production
cargo test -p bcinr-pddl --features mfw-planner --test undeclared_semantics

printf '\n==> External downstream API\n'
cargo test -p bcinr-pddl --features mfw-planner --test downstream_pddl_powl
cargo test -p bcinr-pddl --features mfw-planner --test downstream_cognitive

printf '\n==> Compile every downstream surface\n'
cargo check -p bcinr-pddl --features mfw-planner --all-targets
cargo run -p bcinr-pddl --features mfw-planner --example pddl_to_powl

printf '\nPDDL_TO_POWL_V26_7_24=ALIVE\n'
