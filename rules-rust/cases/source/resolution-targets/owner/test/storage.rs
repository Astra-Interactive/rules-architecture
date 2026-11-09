#[test]
fn resolves_digest() {
    let _digest = crate::digest();
    let _: digest_wire::TestOnly = digest_wire::TestOnly;
}
