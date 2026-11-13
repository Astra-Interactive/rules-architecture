pub(crate) struct SelectBranches {
    pub(crate) futures: Vec<syn::Expr>,
}

impl syn::parse::Parse for SelectBranches {
    fn parse(input: syn::parse::ParseStream<'_>) -> syn::Result<Self> {
        let mut futures = Vec::new();
        while !input.is_empty() {
            if input.peek(syn::Ident)
                && input.peek2(syn::Token![;])
                && input.fork().parse::<syn::Ident>()? == "biased"
            {
                input.parse::<syn::Ident>()?;
                input.parse::<syn::Token![;]>()?;
                continue;
            }
            if input.peek(syn::Token![else]) {
                input.parse::<syn::Token![else]>()?;
                input.parse::<syn::Token![=>]>()?;
                input.parse::<syn::Expr>()?;
            } else {
                input.call(syn::Pat::parse_multi_with_leading_vert)?;
                input.parse::<syn::Token![=]>()?;
                futures.push(input.parse::<syn::Expr>()?);
                if input.peek(syn::Token![,]) {
                    input.parse::<syn::Token![,]>()?;
                    input.parse::<syn::Token![if]>()?;
                    input.parse::<syn::Expr>()?;
                }
                input.parse::<syn::Token![=>]>()?;
                input.parse::<syn::Expr>()?;
            }
            if input.peek(syn::Token![,]) {
                input.parse::<syn::Token![,]>()?;
            }
        }
        Ok(Self { futures })
    }
}
