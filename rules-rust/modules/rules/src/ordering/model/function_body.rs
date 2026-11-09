pub(crate) struct FunctionBody<'syntax> {
    pub(crate) signature: &'syntax syn::Signature,
    pub(crate) block: &'syntax syn::Block,
}
