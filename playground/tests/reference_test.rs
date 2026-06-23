#![feature(min_adt_const_params)]
#![allow(unsafe_code)]

mod reference;

#[test]
fn test_references_compile_and_run() {
    // This is a smoke test to ensure all references compile and their internal unit tests are pulled in.
    assert!(true);
}
