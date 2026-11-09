use crate::check::api::Rule;
use crate::check::model::{RuleContext, RuleFinding};
use crate::module::rule::GlobalStateVisitor;
use crate::module::util::build_shape_item_finding;

pub(crate) struct GlobalState;

impl Rule for GlobalState {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        let mut findings = Vec::new();
        for (ordinal, index) in context.indexes.iter().enumerate() {
            for item in &index.items {
                let syn::Item::Static(shared) = &item.syntax else {
                    continue;
                };
                let mutable = context
                    .resolve_external_type_paths_under(ordinal, item.module, &shared.ty, &item.cfg)
                    .iter()
                    .any(|path| {
                        let name = path.segments.last().map(String::as_str).unwrap_or("");
                        let standard = ["std", "core", "alloc"].contains(&path.package.as_str());
                        let library =
                            ["tokio", "parking_lot", "once_cell"].contains(&path.package.as_str());
                        let state = [
                            "OnceLock", "LazyLock", "Mutex", "RwLock", "Cell", "RefCell",
                            "OnceCell", "Lazy",
                        ]
                        .contains(&name)
                            || name.starts_with("Atomic");
                        (standard || library) && state
                    });
                if mutable {
                    findings.push(build_shape_item_finding(
                        index,
                        item,
                        "GlobalState",
                        "shared state is passed from DI, never stored in a global static",
                    ));
                }
            }
            findings.extend(GlobalStateVisitor::check(context, ordinal));
        }
        findings
    }
}
