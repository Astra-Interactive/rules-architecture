use crate::check::api::Rule;
use crate::check::model::{RuleContext, RuleFinding};
use crate::module::rule::ElapsedReceiverVisitor;
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

    fn reads_wall_clock(
        context: &RuleContext,
        ordinal: usize,
        reference: &crate::source::model::SourceReference,
        elapsed_receivers: &[crate::source::model::NodeSpan],
    ) -> bool {
        let crate::source::model::Resolution::External(path) = &reference.resolution else {
            return false;
        };
        let path = context.resolve_external_identity(ordinal, path);
        let ends_with = |names: &[&str]| {
            path.segments.len() >= names.len()
                && path.segments[path.segments.len() - names.len()..]
                    .iter()
                    .zip(names)
                    .all(|(segment, name)| segment == name)
        };
        let system_now = path.package == "std" && ends_with(&["SystemTime", "now"]);
        let crate_now = match path.package.as_str() {
            "chrono" => ends_with(&["Utc", "now"]) || ends_with(&["Local", "now"]),
            "jiff" => ends_with(&["Timestamp", "now"]) || ends_with(&["Zoned", "now"]),
            "time" => {
                ends_with(&["OffsetDateTime", "now_utc"])
                    || ends_with(&["OffsetDateTime", "now_local"])
                    || ends_with(&["UtcDateTime", "now"])
            }
            _ => false,
        };
        let epoch_elapsed = path.package == "std"
            && ends_with(&["UNIX_EPOCH"])
            && elapsed_receivers.contains(&reference.span);
        system_now || crate_now || epoch_elapsed
    }

    fn is_inside_system_clock(
        context: &RuleContext,
        ordinal: usize,
        reference: &crate::source::model::SourceReference,
    ) -> bool {
        context.indexes[ordinal]
            .implementations
            .iter()
            .any(|implementation| {
                implementation.file == reference.file
                    && implementation.span.contains(&reference.span)
                    && context
                        .resolve_workspace_type(
                            ordinal,
                            implementation.module,
                            &implementation.syntax.self_ty,
                        )
                        .iter()
                        .any(|target| {
                            target.path.item.is_some_and(|identity| {
                                let item = &context.indexes[target.index].items[identity];
                                item.name == "SystemClock"
                                    && matches!(item.syntax, syn::Item::Struct(_))
                            })
                        })
            })
    }

    fn is_production_reference(
        context: &RuleContext,
        ordinal: usize,
        reference: &crate::source::model::SourceReference,
    ) -> bool {
        let production = crate::source::model::CfgPredicate::Not(Box::new(
            crate::source::model::CfgPredicate::Atom {
                key: "test".to_owned(),
                value: None,
            },
        ))
        .in_scope(ordinal);
        context.indexes[ordinal].modules[reference.module].tree
            == crate::source::model::SourceTree::Src
            && reference
                .cfg
                .in_scope(ordinal)
                .and(&production)
                .is_satisfiable()
    }

    fn build_wall_clock_findings(context: &RuleContext) -> Vec<RuleFinding> {
        let mut findings = Vec::new();
        for (ordinal, index) in context.indexes.iter().enumerate() {
            let elapsed_receivers: Vec<_> = context
                .crates
                .iter()
                .filter(|sources| sources.crate_name == index.crate_name)
                .flat_map(|sources| &sources.files)
                .flat_map(|file| ElapsedReceiverVisitor::collect(&file.syntax))
                .collect();
            let mut reads: Vec<_> = index
                .references
                .iter()
                .filter(|reference| {
                    Self::reads_wall_clock(context, ordinal, reference, &elapsed_receivers)
                })
                .filter(|reference| Self::is_production_reference(context, ordinal, reference))
                .filter(|reference| !Self::is_inside_system_clock(context, ordinal, reference))
                .collect();
            reads.sort_by_key(|reference| reference.anchor);
            findings.extend(reads.into_iter().map(|reference| {
                crate::check::util::build_node_finding(
                    "SystemClockOutsideDi",
                    &reference.file,
                    reference.anchor,
                    "SystemTime::now".to_owned(),
                    "only SystemClock reads the wall clock; other code receives a Clock".to_owned(),
                )
            }));
        }
        findings
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
                if misplaced {
                    findings.push(build_shape_item_finding(
                        index,
                        item,
                        "SystemClockOutsideDi",
                        "SystemClock is declared in a system module (time-api.rs.md)",
                    ));
                }
                if signature {
                    findings.push(build_shape_item_finding(
                        index,
                        item,
                        "SystemClockOutsideDi",
                        "the Clock trait declares exactly fn now(&self) -> SystemTime (time-api.rs.md)",
                    ));
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
        findings.extend(Self::build_wall_clock_findings(context));
        findings
    }
}
