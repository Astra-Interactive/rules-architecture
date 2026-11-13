use crate::comment::rule::DocCommentVisitor;

#[test]
fn given_nonstring_doc_attributes_when_collected_then_only_text_is_retained() {
    let syntax = syn::parse_file("#[doc = 1]\n#[doc = source::TEXT]\n#[doc = \"kept\"]\nstruct Value;\ncall!(#[doc=\"macro\"] 1);").expect("attribute syntax");
    let comments = DocCommentVisitor::new().collect_comments(&syntax);
    assert_eq!(comments.len(), 2);
    assert_eq!(comments[0].text, "kept");
    assert_eq!(comments[1].text, "macro");
}
