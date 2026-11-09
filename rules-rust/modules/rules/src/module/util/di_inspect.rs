use crate::check::model::RuleContext;
use crate::module::util::find_kind_name;
use crate::source::model::IndexedItem;

pub(crate) fn is_di_type(context: &RuleContext, ordinal: usize, item: &IndexedItem) -> bool {
    find_kind_name(context, ordinal, item.module) == "di"
        && (item.name.ends_with("Module") || item.name == context.root_module_name())
}

pub(crate) fn resolved_di_type(
    context: &RuleContext,
    target: &crate::source::model::WorkspaceResolvedPath,
) -> bool {
    target.path.item.is_some_and(|identity| {
        is_di_type(
            context,
            target.index,
            &context.indexes[target.index].items[identity],
        )
    })
}

pub(crate) fn is_construction_use(
    context: &RuleContext,
    usage: &crate::source::model::WorkspaceUse,
) -> bool {
    use crate::source::model::WorkspaceUseKind;
    match usage.kind {
        WorkspaceUseKind::Construction => true,
        WorkspaceUseKind::Call => usage.path.last().is_some_and(|name| {
            name == "new"
                || name == "default"
                || name.starts_with("with_")
                || name.starts_with("from_")
        }),
        WorkspaceUseKind::Value => {
            matches!(&context.indexes[usage.target_index].items[usage.target_item].syntax, syn::Item::Struct(value) if matches!(value.fields, syn::Fields::Unit))
        }
        _ => false,
    }
}

fn io_identity(context: &RuleContext, ordinal: usize, path: &str) -> bool {
    let path = context.resolve_external_identity(ordinal, path);
    let standard_io = ["std", "tokio"].contains(&path.package.as_str())
        && path
            .segments
            .get(1)
            .is_some_and(|segment| ["fs", "net"].contains(&segment.as_str()));
    let identity = path
        .segments
        .iter()
        .skip(1)
        .map(String::as_str)
        .collect::<Vec<_>>()
        .join("::");
    standard_io
        && ([
            "fs::File",
            "net::TcpStream",
            "net::TcpListener",
            "net::UdpSocket",
            "net::UnixStream",
            "net::UnixListener",
        ]
        .contains(&identity.as_str())
            || path.segments.last().is_some_and(|name| {
                [
                    "connect",
                    "bind",
                    "read",
                    "read_to_string",
                    "write",
                    "open",
                    "create",
                    "remove_file",
                    "rename",
                    "copy",
                ]
                .contains(&name.as_str())
            }))
}

fn inherent_io_behavior(
    context: &RuleContext,
    ordinal: usize,
    identity: usize,
    implementation: &crate::source::model::IndexedImpl,
    method: &syn::ImplItemFn,
    cfg: &crate::source::model::CfgPredicate,
) -> bool {
    let index = &context.indexes[ordinal];
    let span = crate::source::util::find_node_span(method.block.brace_token.span.join());
    let uses_io = index.references.iter().any(|reference| {
        reference.file == implementation.file && span.contains(&reference.span)
            && reference.cfg.and(cfg).is_satisfiable()
            && matches!(&reference.resolution, crate::source::model::Resolution::External(path) if io_identity(context, ordinal, path))
    });
    let owns_io = match &index.items[identity].syntax {
        syn::Item::Struct(value) => value.fields.iter().any(|field| {
            if !cfg
                .and(&crate::source::model::CfgPredicate::from_attributes(
                    &field.attrs,
                ))
                .is_satisfiable()
            {
                return false;
            }
            context
                .resolve_external_type_paths(ordinal, index.items[identity].module, &field.ty)
                .iter()
                .any(|path| {
                    let name = path.segments.join("::");
                    io_identity(context, ordinal, &name)
                })
        }),
        _ => false,
    };
    uses_io || owns_io
}

fn inherent_dependency(
    context: &RuleContext,
    ordinal: usize,
    identity: usize,
    implementation: &crate::source::model::IndexedImpl,
    cfg: &crate::source::model::CfgPredicate,
) -> bool {
    let index = &context.indexes[ordinal];
    let item = &index.items[identity];
    let service_role = ["Client", "Repository", "Clock", "Gateway", "UseCase"]
        .iter()
        .any(|suffix| item.name.ends_with(suffix))
        || ["service", "usecase"].contains(&find_kind_name(context, ordinal, item.module));
    crate::module::util::owns_implementation(index, identity, implementation)
        && cfg.and(&implementation.cfg).is_satisfiable()
        && implementation.syntax.items.iter().any(|member| {
            let syn::ImplItem::Fn(method) = member else {
                return false;
            };
            let Some(_) = method.sig.receiver() else {
                return false;
            };
            let method_cfg = cfg.and(&implementation.cfg).and(
                &crate::source::model::CfgPredicate::from_attributes(&method.attrs),
            );
            let io_behavior = inherent_io_behavior(
                context,
                ordinal,
                identity,
                implementation,
                method,
                &method_cfg,
            );
            method_cfg.is_satisfiable() && (service_role || io_behavior)
        })
        && !["model", "di"].contains(&find_kind_name(
            context,
            ordinal,
            index.items[identity].module,
        ))
}

pub(crate) fn workspace_implementation_under(
    context: &RuleContext,
    ordinal: usize,
    identity: usize,
    cfg: &crate::source::model::CfgPredicate,
) -> bool {
    let index = &context.indexes[ordinal];
    index.implementations.iter().any(|implementation| {
        if implementation.syntax.trait_.is_none() {
            return inherent_dependency(context, ordinal, identity, implementation, cfg);
        }
        let Some((path, _)) = &implementation.syntax.trait_ else {
            return false;
        };
        crate::module::util::owns_implementation(index, identity, implementation)
            && context
                .resolve_workspace_path(ordinal, implementation.module, path)
                .iter()
                .any(|target| {
                    cfg.in_scope(ordinal)
                        .and(&implementation.cfg.in_scope(ordinal))
                        .and(&target.path.cfg.in_scope(target.index))
                        .is_satisfiable()
                        && target.path.item.is_some_and(|item| {
                            let contract = &context.indexes[target.index].items[item];
                            matches!(contract.syntax, syn::Item::Trait(_))
                                && (find_kind_name(context, target.index, contract.module) == "api"
                                    || [
                                        "Repo",
                                        "Repository",
                                        "Client",
                                        "Clock",
                                        "Gateway",
                                        "Service",
                                    ]
                                    .iter()
                                    .any(|suffix| contract.name.ends_with(suffix)))
                        })
                })
    })
}

pub(crate) fn workspace_implementation(
    context: &RuleContext,
    ordinal: usize,
    identity: usize,
) -> bool {
    workspace_implementation_under(
        context,
        ordinal,
        identity,
        &context.indexes[ordinal].items[identity].cfg,
    )
}

pub(crate) fn lifecycle_trait_path(
    context: &RuleContext,
    ordinal: usize,
    module: usize,
    path: &syn::Path,
) -> bool {
    let configured = context.workspace.root_manifest.root.find(&[
        "workspace",
        "metadata",
        "klibs-rules",
        "lifecycle-traits",
    ]);
    let matches = |name: &str| match configured.map(|value| &value.kind) {
        Some(crate::workspace::model::TomlKind::Array(names)) => names.iter().any(|value| {
            value
                .as_str()
                .is_some_and(|configured| configured.rsplit("::").next() == Some(name))
        }),
        _ => name == "Lifecycle",
    };
    let workspace = context.resolve_workspace_path(ordinal, module, path);
    if !workspace.is_empty() {
        return workspace.iter().any(|target| {
            target.path.item.is_some_and(|identity| {
                matches(&context.indexes[target.index].items[identity].name)
            })
        });
    }
    match context.indexes[ordinal].resolve_path(module, path) {
        crate::source::model::Resolution::External(path) => {
            path.rsplit("::").next().is_some_and(matches)
        }
        _ => false,
    }
}

pub(crate) fn lifecycle_implementation(
    context: &RuleContext,
    ordinal: usize,
    identity: usize,
) -> bool {
    let index = &context.indexes[ordinal];
    index.implementations.iter().any(|implementation| {
        if !crate::module::util::owns_implementation(index, identity, implementation) {
            return false;
        }
        let Some((path, _)) = &implementation.syntax.trait_ else {
            return false;
        };
        lifecycle_trait_path(context, ordinal, implementation.module, path)
    })
}
