#!/usr/bin/env bash
# Cross-language portability proof: build the wasm4games-capi staticlib, link a C harness,
# and verify the C-ABI execution reproduces the native Rust golden corpus digest.
set -euo pipefail
cd "$(dirname "$0")/../.."   # workspace root

echo "[1/3] build staticlib (release; workspace profile = panic=abort)"
cargo build -p wasm4games-capi --release

echo "[2/3] link C harness with cc"
cc crates/wasm4games-capi/harness.c \
   -L target/release -lwasm4games_capi \
   -lpthread -ldl -lm \
   -o target/w4g_harness

echo "[3/3] run cross-language proof"
exec target/w4g_harness
