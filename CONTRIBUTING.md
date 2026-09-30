# Contributing

`CLAUDE.md` is the authoritative development guide. Read it first; this file only covers
what you need before your first pull request.

## Setup

- Rust **nightly** (pinned by `rust-toolchain.toml`, with `rustfmt` and `clippy`).
- `cargo-make` for the `make` targets: `cargo install cargo-make --locked`.
- An external `ggen` binary (providing `graph validate` / `sync`) on `PATH` for
  `make check` and `profile-drift-check`. It is **not** `tools/ggen`. Without it those
  two tasks fail at the `graph-validate` step.

## Before you open a PR

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Conventions

- Conventional commits: `type(scope): description`.
- New algorithm: follow "Add algorithm" in `CLAUDE.md`, including the `pub mod` line in
  `crates/bcinr-logic/src/algorithms/mod.rs`.
- If you change a documented number (test count, unsafe count, crate list), update the docs
  in the same change.
