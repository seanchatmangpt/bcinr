#![cfg(not(miri))]
//! Smoke test that pulls in the `reference` module tree (`petri`, `powl`,
//! `wasm`, `yawl` reference-implementation fixtures) so their own internal
//! `#[cfg(test)]` suites compile and run as part of the workspace test pass.
#![allow(unsafe_code)]

mod reference;

#[test]
fn test_references_compile_and_run() {
    // This is a smoke test to ensure all references compile and their internal unit tests are pulled in.
}
