use crate::check::api::Rule;
use crate::check::model::{RuleContext, RuleFinding};
use crate::module::util::build_shape_item_finding;

pub(crate) struct SystemClockOutsideDi;

impl SystemClockOutsideDi {
    fn valid_clock(
        context: &RuleContext,
        ordinal: usize,
        item: &crate::source::model::IndexedItem,
        contract: &syn::ItemTrait,
    ) -> bool {
        let [syn::TraitItem::Fn(method)] = contract.items.as_slice() else {
            return false;
        };
        let named_now = method.sig.ident == "now";
        let synchronous = method.sig.asyncness.is_none();
        let required = method.default.is_none();
        let single_receiver = method.sig.inputs.len() == 1;
        let nongeneric = method.sig.generics.params.is_empty();
        if !named_now || !synchronous || !required || !single_receiver || !nongeneric {
            return false;
        }
        let Some(receiver) = method.sig.receiver() else {
            return false;
        };
        if !matches!(receiver.kind, syn::ReceiverKind::Reference(_, _, None)) {
            return false;
        }
        let syn::ReturnType::Type(_, typed) = &method.sig.output else {
            return false;
        };
        if context
            .resolve_type_syntax(ordinal, item.module, typed)
            .iter()
            .any(|resolved| !matches!(resolved.typed, syn::Type::Path(_)))
        {
            return false;
        }
        context
            .resolve_outer_external_type_paths(ordinal, item.module, typed)
            .iter()
            .any(|path| {
                path.package == "std"
                    && path
                        .segments
                        .ends_with(&["time".to_owned(), "SystemTime".to_owned()])
            })
    }
}

impl Rule for SystemClockOutsideDi {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        let mut findings = Vec::new();
        for (ordinal, index) in context.indexes.iter().enumerate() {
            for item in &index.items {
                let misplaced = item.name == "SystemClock"
                    && matches!(item.syntax, syn::Item::Struct(_))
                    && !index.modules[item.module]
                        .module
                        .path
                        .iter()
                        .any(|segment| segment == "system");
                let signature = item.name == "Clock"
                    && matches!(&item.syntax, syn::Item::Trait(contract) if !Self::valid_clock(context, ordinal, item, contract));
                if misplaced || signature {
                    findings.push(build_shape_item_finding(index, item, "SystemClockOutsideDi", "SystemClock lives in system and Clock declares exactly fn now(&self) -> SystemTime"));
                }
            }
        }
        let names = context.name_index();
        for usage in names
            .uses
            .iter()
            .filter(|usage| !usage.is_test && !usage.ambiguous && usage.cfg.is_satisfiable())
        {
            let item = &context.indexes[usage.target_index].items[usage.target_item];
            let system_clock =
                item.name == "SystemClock" && matches!(item.syntax, syn::Item::Struct(_));
            let in_di = crate::module::util::find_kind_name(
                context,
                usage.source_index,
                usage.source_module,
            ) == "di";
            let construction = crate::module::util::is_construction_use(context, usage);
            if !system_clock || in_di || !construction {
                continue;
            }
            findings.push(crate::check::util::build_node_finding(
                "SystemClockOutsideDi",
                &usage.file,
                usage.anchor,
                item.name.clone(),
                "construct SystemClock in DI or tests".to_owned(),
            ));
        }
        findings
    }
}
