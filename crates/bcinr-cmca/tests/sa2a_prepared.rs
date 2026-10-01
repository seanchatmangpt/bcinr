use bcinr_cmca::sa2a::*;
#[test]
fn sa2a_prepared() {
    assert_eq!(
        prepared_effect_ref::PreparedEffectRef::exact("s", "e", "d").prepared_digest,
        "d"
    );
}
