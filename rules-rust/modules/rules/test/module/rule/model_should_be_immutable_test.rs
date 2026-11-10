use crate::check::api::Rule;
use crate::module::rule::ModelShouldBeImmutable;
use crate::test::declaration::util::assert_code_findings;
use crate::test::declaration::util::assert_code_fixture;
use crate::test::declaration::util::read_code_context;
use crate::test::module::util::assert_placement_findings;
use crate::test::module::util::read_workspace_shape_context;

#[test]
fn given_an_external_receiver_sharing_a_unique_model_field_when_checked_then_external_assignment_does_not_mutate_model()
 {
    let context = crate::test::source::util::read_resolution_context();
    assert!(
        context
            .indexes
            .iter()
            .any(|index| index.items.iter().any(|item| item.name == "Snapshot"))
    );
    assert!(ModelShouldBeImmutable.check(&context).is_empty());
}

#[test]
fn given_aliased_typed_mutable_self_and_unrelated_type_when_checked_then_reports_only_the_live_model_method()
 {
    let context = read_code_context(
        "mod model { struct Value; }\nmod file { struct Value; impl Value { fn change(&mut self) {} } }\nmod util { type Alias = crate::model::Value; impl Alias { fn consume(mut self) -> Self { self } fn change(self: &mut Self) {} #[cfg(all(target_os = \"linux\", target_os = \"windows\"))] fn inactive(&mut self) {} } }",
    );
    assert_placement_findings(
        &ModelShouldBeImmutable,
        "ModelShouldBeImmutable",
        &context,
        &[("src/lib.rs", 3, "Node")],
    );
}

#[test]
fn given_persistent_placement_cases_when_checked_then_match_exact_node_anchors() {
    assert_code_fixture(&ModelShouldBeImmutable, "ModelShouldBeImmutable", "bad");
    assert_code_fixture(&ModelShouldBeImmutable, "ModelShouldBeImmutable", "good");
}

#[test]
fn given_local_call_returning_model_when_field_is_assigned_then_reports_the_model_write() {
    assert_code_findings(
        &ModelShouldBeImmutable,
        "ModelShouldBeImmutable",
        "mod model { pub struct Invoice { pub total: u64, pub paid: bool } }\n\
         mod memory {\n\
         struct Draft { total: u64 }\n\
         fn build_invoice() -> crate::model::Invoice { crate::model::Invoice { total: 0, paid: false } }\n\
         fn build_draft() -> Draft { Draft { total: 0 } }\n\
         fn adjust_invoice() -> crate::model::Invoice {\n\
         let mut invoice = build_invoice();\n\
         invoice.total = 1;\n\
         invoice\n\
         }\n\
         fn adjust_draft() -> Draft {\n\
         let mut draft = build_draft();\n\
         draft.total = 1;\n\
         draft\n\
         }\n\
         }",
        &[8],
    );
}

#[test]
fn given_vec_push_on_model_field_when_checked_then_reports_the_mutation() {
    assert_code_findings(
        &ModelShouldBeImmutable,
        "ModelShouldBeImmutable",
        "mod model { pub struct Invoice { pub revision: u32, pub entries: Vec<u32> } }\n\
         mod memory {\n\
         struct Journal { entries: Vec<u32> }\n\
         fn append_invoice_entry(mut invoice: crate::model::Invoice) -> crate::model::Invoice {\n\
         invoice.entries.push(1);\n\
         invoice\n\
         }\n\
         fn append_journal_entry(mut journal: Journal) -> Journal {\n\
         journal.entries.push(1);\n\
         journal\n\
         }\n\
         fn count_entries(invoice: &crate::model::Invoice) -> usize { invoice.entries.len() }\n\
         }",
        &[5],
    );
}

#[test]
fn given_model_write_in_match_guard_when_checked_then_reports_the_assignment() {
    assert_code_findings(
        &ModelShouldBeImmutable,
        "ModelShouldBeImmutable",
        "mod model { pub struct Invoice { pub total: u64, pub paid: bool } }\n\
         mod util {\n\
         impl crate::model::Invoice {\n\
         fn inspect(mut self, flag: bool) -> Self {\n\
         match flag {\n\
         _ if { self.total = 1; true } => self,\n\
         _ => self,\n\
         }\n\
         }\n\
         }\n\
         }",
        &[6],
    );
}

#[test]
fn given_model_write_in_nested_fn_when_checked_then_reports_the_assignment() {
    assert_code_findings(
        &ModelShouldBeImmutable,
        "ModelShouldBeImmutable",
        "mod model { pub struct Invoice { pub total: u64, pub paid: bool } }\n\
         mod memory {\n\
         struct Draft { total: u64 }\n\
         fn update_invoice(invoice: crate::model::Invoice) -> crate::model::Invoice {\n\
         fn change(mut value: crate::model::Invoice) -> crate::model::Invoice {\n\
         value.total = 1;\n\
         value\n\
         }\n\
         fn touch(mut draft: Draft) -> Draft {\n\
         draft.total = 1;\n\
         draft\n\
         }\n\
         touch(Draft { total: 0 });\n\
         change(invoice)\n\
         }\n\
         }",
        &[6],
    );
}

#[test]
fn given_model_wrapper_when_wrapper_field_is_written_then_only_model_write_reports() {
    assert_code_findings(
        &ModelShouldBeImmutable,
        "ModelShouldBeImmutable",
        "mod model { pub struct Invoice { pub total: u64, pub paid: bool } }\n\
         mod memory {\n\
         pub struct Revisioned<T> { pub value: T, pub revision: u64 }\n\
         fn bump_invoice_revision(entry: &mut Revisioned<crate::model::Invoice>) {\n\
         entry.revision += 1;\n\
         }\n\
         fn rewrite_invoice_total(invoice: &mut crate::model::Invoice) {\n\
         invoice.total = 2;\n\
         }\n\
         }",
        &[8],
    );
}

#[test]
fn given_model_in_generic_wrapper_when_its_field_is_assigned_then_reports_the_write() {
    assert_code_findings(
        &ModelShouldBeImmutable,
        "ModelShouldBeImmutable",
        "mod model { pub struct Invoice { pub total: u64, pub paid: bool } }\n\
         mod memory {\n\
         pub struct Revisioned<T> { pub value: T, pub revision: u64 }\n\
         pub struct InvoiceEntry { pub value: crate::model::Invoice, pub revision: u64 }\n\
         fn settle_stored_invoice(entry: &mut Revisioned<crate::model::Invoice>) {\n\
         entry.value.paid = true;\n\
         }\n\
         fn settle_plain_entry(entry: &mut InvoiceEntry) {\n\
         entry.value.paid = true;\n\
         }\n\
         }",
        &[6, 9],
    );
}

#[test]
fn given_model_loop_element_when_field_is_assigned_then_reports_only_the_model_write() {
    assert_code_findings(
        &ModelShouldBeImmutable,
        "ModelShouldBeImmutable",
        "mod model { pub struct Invoice { pub total: u64, pub paid: bool } }\n\
         mod memory {\n\
         struct Draft { total: u64 }\n\
         fn settle_invoices(mut invoices: Vec<crate::model::Invoice>) -> Vec<crate::model::Invoice> {\n\
         for invoice in &mut invoices {\n\
         invoice.paid = true;\n\
         }\n\
         invoices\n\
         }\n\
         fn settle_drafts(mut drafts: Vec<Draft>) -> Vec<Draft> {\n\
         for draft in &mut drafts {\n\
         draft.total = 1;\n\
         }\n\
         drafts\n\
         }\n\
         }",
        &[6],
    );
}

#[test]
fn given_destructured_model_when_field_is_assigned_then_reports_only_the_model_write() {
    assert_code_findings(
        &ModelShouldBeImmutable,
        "ModelShouldBeImmutable",
        "mod model { pub struct Invoice { pub total: u64, pub paid: bool } }\n\
         mod memory {\n\
         struct Draft { total: u64 }\n\
         fn settle_first(first: Option<crate::model::Invoice>) -> Option<crate::model::Invoice> {\n\
         match first {\n\
         Some(mut invoice) => {\n\
         invoice.paid = true;\n\
         Some(invoice)\n\
         }\n\
         None => None,\n\
         }\n\
         }\n\
         fn settle_draft(first: Option<Draft>) -> Option<Draft> {\n\
         match first {\n\
         Some(mut draft) => {\n\
         draft.total = 1;\n\
         Some(draft)\n\
         }\n\
         None => None,\n\
         }\n\
         }\n\
         }",
        &[7],
    );
}

#[test]
fn given_index_write_into_model_field_when_checked_then_reports_both_assignments() {
    assert_code_findings(
        &ModelShouldBeImmutable,
        "ModelShouldBeImmutable",
        "mod model { pub struct Invoice { pub revision: u32, pub entries: Vec<u32> } }\n\
         mod memory {\n\
         struct Journal { entries: Vec<u32> }\n\
         fn set_first_entry(mut invoice: crate::model::Invoice) -> crate::model::Invoice {\n\
         invoice.entries[0] = 1;\n\
         invoice\n\
         }\n\
         fn raise_first_entry(mut invoice: crate::model::Invoice) -> crate::model::Invoice {\n\
         invoice.entries[0] += 1;\n\
         invoice\n\
         }\n\
         fn set_first_journal_entry(mut journal: Journal) -> Journal {\n\
         journal.entries[0] = 1;\n\
         journal\n\
         }\n\
         }",
        &[5, 9],
    );
}

#[test]
fn given_ext_trait_on_foreign_crate_model_when_checked_then_reports_mutable_method() {
    let context = read_workspace_shape_context(&[
        (
            "shop_billing_api",
            "",
            "pub mod model { #[derive(Clone)] pub struct Invoice { pub total: u64, pub paid_at: Option<std::time::SystemTime> } }",
        ),
        (
            "shop_billing_impl",
            "shop_billing_api={path=\"../shop_billing_api\"}",
            "mod util {\n\
             use shop_billing_api::model::Invoice;\n\
             pub trait InvoiceExt {\n\
             fn settle(&mut self, now: std::time::SystemTime);\n\
             fn is_settled(&self) -> bool;\n\
             }\n\
             impl InvoiceExt for Invoice {\n\
             fn settle(&mut self, now: std::time::SystemTime) {\n\
             *self = Invoice { paid_at: Some(now), ..self.clone() };\n\
             }\n\
             fn is_settled(&self) -> bool { self.paid_at.is_some() }\n\
             }\n\
             }",
        ),
    ]);
    let lines: Vec<Option<u32>> = ModelShouldBeImmutable
        .check(&context)
        .iter()
        .map(|finding| finding.finding.location.line)
        .collect();
    assert_eq!(vec![Some(8)], lines);
}

#[test]
fn given_pinned_mutable_self_receiver_on_model_when_checked_then_reports_only_that_method() {
    assert_code_findings(
        &ModelShouldBeImmutable,
        "ModelShouldBeImmutable",
        "mod model { pub struct Pending(u8); }\n\
         mod util {\n\
         impl crate::model::Pending {\n\
         fn advance(self: std::pin::Pin<&mut Self>) {}\n\
         fn peek(self: std::pin::Pin<&Self>) {}\n\
         fn finish(self: Box<Self>) {}\n\
         fn consume(mut self) -> Self { self }\n\
         fn read(&self) {}\n\
         }\n\
         }",
        &[4],
    );
}
