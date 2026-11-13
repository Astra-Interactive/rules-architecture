use crate::source::model::Comment;
use crate::source::model::CommentKind;
use crate::source::model::TextPosition;
use crate::source::util::parse_macro_arguments;
use syn::Attribute;
use syn::Expr;
use syn::Lit;
use syn::Meta;
use syn::spanned::Spanned;
use syn::visit::Visit;

pub(crate) struct DocCommentVisitor {
    comments: Vec<Comment>,
}

impl DocCommentVisitor {
    pub(crate) fn new() -> Self {
        Self {
            comments: Vec::new(),
        }
    }

    pub(crate) fn collect_comments(mut self, syntax: &syn::File) -> Vec<Comment> {
        self.visit_file(syntax);
        self.comments
    }
}

impl<'ast> Visit<'ast> for DocCommentVisitor {
    fn visit_macro(&mut self, item: &'ast syn::Macro) {
        for argument in &parse_macro_arguments(item) {
            self.visit_expr(argument);
        }
    }

    fn visit_attribute(&mut self, attribute: &'ast Attribute) {
        let Meta::NameValue(name_value) = &attribute.meta else {
            return;
        };
        if !name_value.path.is_ident("doc") {
            return;
        }
        let Expr::Lit(literal) = &name_value.value else {
            return;
        };
        let Lit::Str(text) = &literal.lit else {
            return;
        };
        let start = attribute.span().start();
        self.comments.push(Comment {
            start: TextPosition {
                line: u32::try_from(start.line).unwrap_or(u32::MAX),
                column: u32::try_from(start.column).unwrap_or(u32::MAX),
            },
            text: text.value(),
            kind: CommentKind::Line,
        });
    }
}
