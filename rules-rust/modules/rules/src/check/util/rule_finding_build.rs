use crate::check::model::Anchor;
use crate::check::model::Finding;
use crate::check::model::Location;
use crate::check::model::RuleFinding;
use crate::check::model::RuleId;
use crate::source::model::NodeSpan;
use syn::ItemMod;
use syn::ext::IdentExt;

pub(crate) fn build_node_finding(
    rule_id: &str,
    file: &str,
    span: NodeSpan,
    subject: String,
    message: String,
) -> RuleFinding {
    RuleFinding {
        finding: Finding {
            rule_id: RuleId(rule_id.to_owned()),
            location: Location {
                path: file.to_owned(),
                line: Some(span.start.line),
            },
            subject,
            message,
        },
        anchor: Anchor::Node {
            file: file.to_owned(),
            span,
        },
    }
}

pub(crate) fn build_child_module_path(parent: &str, item: &ItemMod) -> String {
    format!("{parent}::{}", item.ident.unraw())
}

pub(crate) fn format_module_path(crate_name: &str, path: &[String]) -> String {
    let mut segments = vec![crate_name.replace('-', "_")];
    segments.extend(path.iter().cloned());
    segments.join("::")
}
