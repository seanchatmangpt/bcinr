# Security Policy

## Supported versions

Only the latest released version (currently `26.9.28`) receives security fixes.
Releases are CalVer (`YY.M.D`); a fix ships as the next release.

## Reporting a vulnerability

Please do not open a public issue. Use GitHub's private vulnerability reporting:
**Security → Report a vulnerability** on <https://github.com/seanchatmangpt/bcinr>.

Include the affected crate and version, a minimal reproduction, and the impact you
expect. Expect an acknowledgement within a few days.

## Known exposure

A revoked Ed25519 signing key is reachable in git history. The details and the
accepted-risk decision are in `docs/security/key-exposure-v26.9.25.md`. Nothing in this
repository should trust that key.

## Scope notes

- `bcinr-cmca` ships a deprecated authority chain (CMCA-102 / CMCA-114) that has not
  completed Hoare-logic verification. Do not rely on it for security decisions.
- `bcinr-powl` and `bcinr-pddl` require a nightly Rust toolchain.
