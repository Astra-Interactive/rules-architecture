use case_invoice::invoice::util::load_invoice;

#[test]
fn given_invoice_when_loaded_then_present() {
    assert_eq!(1, load_invoice());
}
