pub mod check {
    pub mod api {
        pub use report_writer::ReportWriter;
        pub(crate) use rule::Rule;
        mod report_writer;
        mod rule;
    }
    pub mod model {
        pub use accepted_exception::AcceptedException;
        pub(crate) use anchor::Anchor;
        pub use api_ownership_report::ApiOwnershipReport;
        pub use check_error::CheckError;
        pub use checked_workspace::CheckedWorkspace;
        pub use finding::Finding;
        pub use location::Location;
        pub use report::Report;
        pub(crate) use rule_context::RuleContext;
        pub(crate) use rule_finding::RuleFinding;
        pub use rule_id::RuleId;
        mod accepted_exception;
        mod anchor;
        mod api_ownership_report;
        mod check_error;
        mod checked_workspace;
        mod finding;
        mod location;
        mod report;
        mod rule_context;
        mod rule_finding;
        mod rule_id;
    }
    pub mod usecase {
        pub use check_workspace_use_case::CheckWorkspaceUseCase;
        mod check_workspace_use_case;
    }
    pub(crate) mod util {
        pub(crate) use api_ownership_build::build_api_ownership;
        pub(crate) use report_build::build_report;
        pub(crate) use rule_finding_build::build_child_module_path;
        pub(crate) use rule_finding_build::build_node_finding;
        pub(crate) use rule_finding_build::format_module_path;
        pub(crate) use unanalysable_findings::build_unanalysable_findings;
        mod api_ownership_build;
        mod report_build;
        mod report_ext;
        mod rule_context_ext;
        mod rule_finding_build;
        mod unanalysable_findings;
        mod workspace_type_query;
    }
    pub(crate) mod file {
        pub(crate) use file_report_writer::FileReportWriter;
        mod file_report_writer;
    }
    pub mod di {
        pub use check_module::CheckModule;
        pub(crate) use rule_module::RuleModule;
        mod check_module;
        mod rule_module;
    }
}

pub mod config {
    pub mod model {
        pub(crate) use allowed_dependency::AllowedDependency;
        pub(crate) use carrier_lint::CarrierLint;
        pub(crate) use crate_config::CrateConfig;
        pub(crate) use manifest_exception::ManifestException;
        pub(crate) use manifest_exception::ManifestScope;
        pub(crate) use project_config::ProjectConfig;
        pub use required_configuration::RequiredConfiguration;
        pub(crate) use role::Role;
        mod allowed_dependency;
        mod carrier_lint;
        mod crate_config;
        mod manifest_exception;
        mod project_config;
        mod required_configuration;
        mod role;
    }
    pub mod util {
        pub(crate) use checks_parse::parse_allowed_dependencies;
        pub(crate) use checks_parse::parse_disabled_checks;
        pub(crate) use config_parse::parse_project_config;
        pub(crate) use config_value::build_configuration_error;
        pub(crate) use config_value::check_known_keys;
        pub(crate) use config_value::parse_reason;
        pub(crate) use config_value::parse_string;
        pub(crate) use config_value::parse_string_list;
        pub use required_configuration_render::render_required_configuration;
        pub(crate) use role_resolve::detect_role;
        pub(crate) use role_resolve::format_role_name;
        pub(crate) use role_resolve::list_role_names;
        pub(crate) use role_resolve::parse_role;
        mod checks_parse;
        mod config_parse;
        mod config_value;
        mod required_configuration_render;
        mod role_resolve;
    }
    pub(crate) mod rule {
        pub(crate) use lint_config::LintConfig;
        mod lint_config;
    }

    pub(crate) mod di {
        pub(crate) use config_rule_module::ConfigRuleModule;
        mod config_rule_module;
    }
}

pub(crate) mod workspace {
    pub(crate) mod api {
        pub(crate) use rustfmt_reader::RustfmtReader;
        pub(crate) use workspace_reader::WorkspaceReader;
        mod rustfmt_reader;
        mod workspace_reader;
    }
    pub(crate) mod model {
        pub(crate) use dependency::Dependency;
        pub(crate) use dependency::DependencyKind;
        pub(crate) use lock_package::LockPackage;
        pub(crate) use lock_reference::LockReference;
        pub(crate) use member::Member;
        pub(crate) use rustfmt_facts::RustfmtFacts;
        pub(crate) use target::Target;
        pub(crate) use toml_file::TomlFile;
        pub(crate) use toml_value::TomlEntry;
        pub(crate) use toml_value::TomlKind;
        pub(crate) use toml_value::TomlValue;
        pub(crate) use workspace::ToolchainFile;
        pub(crate) use workspace::Workspace;
        mod dependency;
        mod lock_package;
        mod lock_reference;
        mod member;
        mod rustfmt_facts;
        mod target;
        mod toml_file;
        mod toml_value;
        mod workspace;
    }
    pub(crate) mod util {
        pub(crate) use dependency_identity::resolve_patched_dependency;
        pub(crate) use directory_skip::is_member_source_directory;
        pub(crate) use directory_skip::is_skipped_directory_name;
        pub(crate) use lock_dependency::build_lock_error;
        pub(crate) use lock_dependency::select_locked_dependency;
        pub(crate) use toml_file_parse::parse_toml_file;
        pub(crate) use toolchain_path::resolve_cargo_path;
        pub(crate) use toolchain_path::resolve_rustfmt_path;
        mod dependency_ext;
        mod dependency_identity;
        mod directory_skip;
        mod lock_dependency;
        mod member_ext;
        mod target_ext;
        mod toml_file_parse;
        mod toml_value_ext;
        mod toolchain_path;
    }
    pub(crate) mod cargo {
        pub(crate) use cargo_rustfmt_reader::CargoRustfmtReader;
        pub(crate) use cargo_workspace_reader::CargoWorkspaceReader;
        mod cargo_rustfmt_reader;
        mod cargo_workspace_reader;
    }
}

pub(crate) mod source {
    pub(crate) mod api {
        pub(crate) use source_reader::SourceReader;
        mod source_reader;
    }
    pub(crate) mod model {
        pub(crate) use cfg_predicate::CfgPredicate;
        pub(crate) use comment::Comment;
        pub(crate) use comment::CommentKind;
        pub(crate) use crate_index::{
            CrateIndex, ImportBinding, IndexedImpl, IndexedItem, IndexedModule, Resolution,
            ResolvedPath, SourceEdge,
        };
        pub(crate) use crate_sources::CrateSources;
        pub(crate) use expectation::Expectation;
        pub(crate) use expectation::ExpectationPosition;
        pub(crate) use external_path::ExternalPath;
        pub(crate) use macro_body::MacroBody;
        pub(crate) use module_frame::ModuleFrame;
        pub(crate) use module_id::ModuleId;
        pub(crate) use node_span::NodeSpan;
        pub(crate) use reference_site::ReferenceSite;
        pub(crate) use resolved_type_syntax::ResolvedTypeSyntax;
        pub(crate) use select_part::SelectPart;
        pub(crate) use source_file::SourceFile;
        pub(crate) use source_load_error::SourceLoadError;
        pub(crate) use source_module::DeclaredVisibility;
        pub(crate) use source_module::ModuleDeclaration;
        pub(crate) use source_module::SourceModule;
        pub(crate) use source_reference::SourceReference;
        pub(crate) use source_tree::SourceTree;
        pub(crate) use text_position::TextPosition;
        pub(crate) use type_binding::TypeBinding;
        pub(crate) use unanalysable_source::UnanalysableSite;
        pub(crate) use unanalysable_source::UnanalysableSource;
        pub(crate) use workspace_name_index::WorkspaceNameIndex;
        pub(crate) use workspace_resolved_path::WorkspaceResolvedPath;
        pub(crate) use workspace_type_result::WorkspaceTypeResult;
        pub(crate) use workspace_use::{WorkspaceUse, WorkspaceUseKind};
        mod cfg_predicate;
        mod comment;
        mod crate_index;
        mod crate_sources;
        mod expectation;
        mod external_path;
        mod macro_body;
        mod module_frame;
        mod module_id;
        mod node_span;
        mod reference_site;
        mod resolved_type_syntax;
        mod select_part;
        mod source_file;
        mod source_load_error;
        mod source_module;
        mod source_reference;
        mod source_tree;
        mod text_position;
        mod type_binding;
        mod unanalysable_source;
        mod workspace_name_index;
        mod workspace_resolved_path;
        mod workspace_type_result;
        mod workspace_use;
    }
    pub(crate) mod util {
        pub(crate) use comment_scan::scan_comments;
        pub(crate) use derive_attribute::list_derive_paths;
        pub(crate) use expression_attribute::find_expression_attributes;
        pub(crate) use macro_arguments::parse_macro_arguments;
        pub(crate) use macro_arguments::parse_macro_expressions;
        pub(crate) use macro_arguments::parse_select_parts;
        pub(crate) use macro_arguments::visit_macro_body;
        pub(crate) use select_arguments::parse_select_arguments;
        pub(crate) use syntax_record::find_declared_visibility;
        pub(crate) use syntax_record::find_node_span;
        pub(crate) use syntax_record::find_path_attribute;
        pub(crate) use syntax_record::format_tokens;
        pub(crate) use syntax_record::has_conditional_path;
        pub(crate) use target_predicate::find_single_target_value;
        pub(crate) use target_predicate::list_manifest_target_values;
        pub(crate) use target_predicate::list_target_values;
        mod cfg_predicate_ext;
        mod comment_scan;
        mod crate_index_ext;
        mod derive_attribute;
        mod expression_attribute;
        mod external_path_ext;
        mod macro_arguments;
        mod node_span_ext;
        mod select_arguments;
        mod syntax_record;
        mod target_predicate;
        mod workspace_name_index_ext;
    }
    pub(crate) mod syn {
        pub(crate) use alias_syntax_folder::AliasSyntaxFolder;
        pub(crate) use crate_index_builder::CrateIndexBuilder;
        pub(crate) use expectation_parser::ExpectationParser;
        pub(crate) use expectation_visitor::ExpectationVisitor;
        pub(crate) use module_tree_walker::ModuleTreeWalker;
        pub(crate) use name_availability_visitor::NameAvailabilityVisitor;
        pub(crate) use path_edge_visitor::PathEdgeVisitor;
        pub(crate) use reference_site_visitor::ReferenceSiteVisitor;
        pub(crate) use syn_source_reader::SynSourceReader;
        pub(crate) use workspace_type_resolver::WorkspaceTypeResolver;
        mod alias_syntax_folder;
        mod crate_index_builder;
        mod expectation_parser;
        mod expectation_visitor;
        mod module_tree_walker;
        mod name_availability_visitor;
        mod path_edge_visitor;
        mod reference_site_visitor;
        mod syn_source_reader;
        mod workspace_type_resolver;
    }
}

pub(crate) mod suppression {
    pub(crate) mod rule {
        pub(crate) use expect_scope::ExpectScope;
        pub(crate) use lint_expectation_visitor::LintExpectationVisitor;
        mod expect_scope;
        mod lint_expectation_visitor;
    }
    pub(crate) mod model {
        pub(crate) use expectation_slot::ExpectationSlot;
        pub(crate) use rule_activity::RuleActivity;
        pub(crate) use suppression_outcome::SuppressionOutcome;
        pub(crate) use suppressor::Suppressor;
        pub(crate) use validated_expectations::ValidatedExpectations;
        mod expectation_slot;
        mod rule_activity;
        mod suppression_outcome;
        mod suppressor;
        mod validated_expectations;
    }
    pub(crate) mod util {
        pub(crate) use expectation_validate::validate_expectations;
        pub(crate) use meta_finding::build_meta_finding;
        pub(crate) use suppression_apply::apply_suppressions;
        mod expectation_validate;
        mod meta_finding;
        mod suppression_apply;
    }

    pub(crate) mod di {
        pub(crate) use suppression_rule_module::SuppressionRuleModule;
        mod suppression_rule_module;
    }
}

pub(crate) mod catalogue {
    pub(crate) mod model {
        pub(crate) use catalogue_entry::AnchorKinds;
        pub(crate) use catalogue_entry::CatalogueEntry;
        pub(crate) use retired_rule::RetiredRule;
        mod catalogue_entry;
        mod retired_rule;
    }
    pub(crate) mod util {
        pub(crate) use rule_catalogue::find_catalogue_entry;
        pub(crate) use rule_catalogue::find_retired_rule;
        mod rule_catalogue;
    }
}

pub(crate) mod comment {
    pub(crate) mod rule {
        pub(crate) use doc_comment_scope::DocCommentScope;
        pub(crate) use doc_comment_visitor::DocCommentVisitor;
        pub(crate) use doc_link_visitor::DocLinkVisitor;
        pub(crate) use doc_scope_visitor::DocScopeVisitor;
        pub(crate) use forbidden_comment::ForbiddenComment;
        pub(crate) use intra_doc_link::IntraDocLink;
        pub(crate) use panic_doc_visitor::PanicDocVisitor;
        pub(crate) use rustdoc_filler::RustdocFiller;
        pub(crate) use rustdoc_html_tag::RustdocHtmlTag;
        pub(crate) use rustdoc_parameter_visitor::RustdocParameterVisitor;
        mod doc_comment_scope;
        mod doc_comment_visitor;
        mod doc_link_visitor;
        mod doc_scope_visitor;
        mod forbidden_comment;
        mod intra_doc_link;
        mod panic_doc_visitor;
        mod rustdoc_filler;
        mod rustdoc_html_tag;
        mod rustdoc_parameter_visitor;
    }
    pub(crate) mod model {
        pub(crate) use doc_owner::DocOwner;
        mod doc_owner;
    }
    pub(crate) mod util {
        pub(crate) use comment_shape::find_forbidden_shape;
        pub(crate) use doc_attributes::{
            doc_attribute_lines, doc_heading, item_doc_attributes, joined_doc,
        };
        pub(crate) use doc_examples::collect_invalid_example_sections;
        pub(crate) use doc_inline_code::doc_inline_candidates;
        pub(crate) use doc_resolve::doc_resolves;
        pub(crate) use doc_text::strip_doc_code;
        mod comment_shape;
        mod doc_attributes;
        mod doc_examples;
        mod doc_inline_code;
        mod doc_resolve;
        mod doc_text;
    }

    pub(crate) mod di {
        pub(crate) use comment_rule_module::CommentRuleModule;
        mod comment_rule_module;
    }
}

pub(crate) mod import {
    pub(crate) mod rule {
        pub(crate) use glob_import_visitor::GlobImportVisitor;
        pub(crate) use import_order_visitor::ImportOrderVisitor;
        pub(crate) use import_ordering::ImportOrdering;
        pub(crate) use wildcard_import::WildcardImport;
        mod glob_import_visitor;
        mod import_order_visitor;
        mod import_ordering;
        mod wildcard_import;
    }

    pub(crate) mod di {
        pub(crate) use import_rule_module::ImportRuleModule;
        mod import_rule_module;
    }
}

pub(crate) mod naming {
    pub(crate) mod rule {
        pub(crate) use closure_parameter_name::ClosureParameterName;
        pub(crate) use explicit_it_name::ExplicitItName;
        pub(crate) use function_verb::FunctionVerb;
        pub(crate) use implementation_name::ImplementationName;
        pub(crate) use kind_name_in_package::KindNameInPackage;
        pub(crate) use name_visitor::NameVisitor;
        pub(crate) use roleless_type_name::RolelessTypeName;
        pub(crate) use test_function_name::TestFunctionName;
        mod closure_parameter_name;
        mod explicit_it_name;
        mod function_verb;
        mod implementation_name;
        mod kind_name_in_package;
        mod name_visitor;
        mod roleless_type_name;
        mod test_function_name;
    }

    pub(crate) mod di {
        pub(crate) use naming_rule_module::NamingRuleModule;
        mod naming_rule_module;
    }
}

pub(crate) mod ordering {
    pub(crate) mod rule {
        pub(crate) use called_function_visitor::CalledFunctionVisitor;
        pub(crate) use callee_before_caller::CalleeBeforeCaller;
        pub(crate) use declaration_order::DeclarationOrder;
        mod called_function_visitor;
        mod callee_before_caller;
        mod declaration_order;
    }
    pub(crate) mod model {
        pub(crate) use family_order::FamilyOrder;
        pub(crate) use function_body::FunctionBody;
        mod family_order;
        mod function_body;
    }

    pub(crate) mod di {
        pub(crate) use ordering_rule_module::OrderingRuleModule;
        mod ordering_rule_module;
    }
}

pub(crate) mod family {
    pub(crate) mod model {
        pub(crate) use type_behaviour::TypeBehaviour;
        pub(crate) use type_declaration::TypeDeclaration;
        mod type_behaviour;
        mod type_declaration;
    }
    pub(crate) mod rule {
        pub(crate) use constant_placement::ConstantPlacement;
        pub(crate) use one_type_family_per_file::OneTypeFamilyPerFile;
        mod constant_placement;
        mod one_type_family_per_file;
    }
    pub(crate) mod util {
        pub(crate) use type_family::{find_primary_type, uses_type};
        mod type_family;
    }

    pub(crate) mod di {
        pub(crate) use family_rule_module::FamilyRuleModule;
        mod family_rule_module;
    }
}

pub(crate) mod default {
    pub(crate) mod rule {
        pub(crate) use struct_default_value::StructDefaultValue;
        pub(crate) use struct_update_visitor::StructUpdateVisitor;
        mod struct_default_value;
        mod struct_update_visitor;
    }

    pub(crate) mod di {
        pub(crate) use default_rule_module::DefaultRuleModule;
        mod default_rule_module;
    }
}

pub(crate) mod syntax {
    pub(crate) mod util {
        pub(crate) use item_shape::{
            find_self_name, find_type_name, is_accessor_adapter, is_constructor, to_snake_case,
        };
        mod item_shape;
    }
}

pub(crate) mod layout {
    pub(crate) mod rule {
        pub(crate) use container_sources::ContainerSources;
        pub(crate) use crate_targets::CrateTargets;
        pub(crate) use nested_unit::NestedUnit;
        pub(crate) use platform_source_set_unit::PlatformSourceSetUnit;
        pub(crate) use plugin_impl_unit::PluginImplUnit;
        pub(crate) use project_dir_matches_path::ProjectDirMatchesPath;
        pub(crate) use published_version::PublishedVersion;
        pub(crate) use root_build_subprojects::RootBuildSubprojects;
        pub(crate) use root_package_path::RootPackagePath;
        pub(crate) use root_sources::RootSources;
        pub(crate) use shell_automation::ShellAutomation;
        pub(crate) use unit_path::UnitPath;
        pub(crate) use xtask_alias::XtaskAlias;
        mod container_sources;
        mod crate_targets;
        mod nested_unit;
        mod platform_source_set_unit;
        mod plugin_impl_unit;
        mod project_dir_matches_path;
        mod published_version;
        mod root_build_subprojects;
        mod root_package_path;
        mod root_sources;
        mod shell_automation;
        mod unit_path;
        mod xtask_alias;
    }

    pub(crate) mod di {
        pub(crate) use layout_rule_module::LayoutRuleModule;
        mod layout_rule_module;
    }
}

pub(crate) mod dependency {
    pub(crate) mod util {
        pub(crate) use role_dependency::build_dependency_finding;
        mod role_dependency;
    }
    pub(crate) mod rule {
        pub(crate) use api_unit_dependency::ApiUnitDependency;
        pub(crate) use api_unit_platform_dependency::ApiUnitPlatformDependency;
        pub(crate) use dependency_floor::DependencyFloor;
        pub(crate) use entry_point_dependency::EntryPointDependency;
        pub(crate) use platform_dependency_table::PlatformDependencyTable;
        pub(crate) use technology_dependency::TechnologyDependency;
        pub(crate) use time_crate::TimeCrate;
        pub(crate) use unit_dependency::UnitDependency;
        pub(crate) use unit_dependency_cycle::UnitDependencyCycle;
        mod api_unit_dependency;
        mod api_unit_platform_dependency;
        mod dependency_floor;
        mod entry_point_dependency;
        mod platform_dependency_table;
        mod technology_dependency;
        mod time_crate;
        mod unit_dependency;
        mod unit_dependency_cycle;
    }

    pub(crate) mod di {
        pub(crate) use dependency_rule_module::DependencyRuleModule;
        mod dependency_rule_module;
    }
}

pub(crate) mod module {
    pub(crate) mod model {
        pub(crate) use cycle_component::CycleComponent;
        pub(crate) use cycle_search_step::CycleSearchStep;
        pub(crate) use hidden_member::HiddenMember;
        pub(crate) use lifecycle_facts::LifecycleFacts;
        pub(crate) use module_dependency_path::ModuleDependencyPath;
        pub(crate) use module_identity::ModuleIdentity;
        pub(crate) use visibility_member::VisibilityMember;
        mod cycle_component;
        mod cycle_search_step;
        mod hidden_member;
        mod lifecycle_facts;
        mod module_dependency_path;
        mod module_identity;
        mod visibility_member;
    }
    pub(crate) mod util {
        pub(crate) use di_inspect::{
            is_construction_use, is_di_type, lifecycle_implementation, lifecycle_trait_path,
            resolved_di_type, workspace_implementation, workspace_implementation_under,
        };
        pub(crate) use lifecycle_graph::has_module_parameter_cycle;
        pub(crate) use placement_inspect::{
            external_trait, is_model_item, item_concept, owns_implementation, reference_types,
            resolve_type_uses,
        };
        pub(crate) use returned_type_use::returned_to_other_owner;
        pub(crate) use shape_inspect::{
            build_shape_item_finding, build_shape_module_finding, find_kind_name,
            find_shape_visibility, find_type_path, implements_crate_trait, list_shape_words,
            resolve_shape_type,
        };
        pub(crate) use type_ownership::{
            declaration_owner, exclusive_field_owner, field_owners, has_type_behaviour,
            is_used_beside_its_field_owner,
        };
        mod external_identity;
        mod kind_resolve;
        mod placement_inspect;
        mod registration_base;
        mod registration_shape;
        mod returned_type_use;
        mod shape_inspect;
        mod type_ownership;
        mod workspace_visibility;

        mod di_inspect;
        mod lifecycle_graph;
        mod rule_context_ext;
    }
    pub(crate) mod rule {
        pub(crate) use api_configuration::ApiConfiguration;
        pub(crate) use associated_factory_function::AssociatedFactoryFunction;
        pub(crate) use constructor_dependency::ConstructorDependency;
        pub(crate) use contract_placement::ContractPlacement;
        pub(crate) use contract_unit_shape::ContractUnitShape;
        pub(crate) use core_shape::CoreShape;
        pub(crate) use di_module_lifecycle::DiModuleLifecycle;
        pub(crate) use di_module_shape::DiModuleShape;
        pub(crate) use di_module_visibility::DiModuleVisibility;
        pub(crate) use di_placement::DiPlacement;
        pub(crate) use doc_hidden_use::DocHiddenUse;
        pub(crate) use doc_hidden_visitor::DocHiddenVisitor;
        pub(crate) use elapsed_receiver_visitor::ElapsedReceiverVisitor;
        pub(crate) use entry_point_shape::EntryPointShape;
        pub(crate) use entry_point_visibility::EntryPointVisibility;
        pub(crate) use enum_family_file::EnumFamilyFile;
        pub(crate) use error_placement::ErrorPlacement;
        pub(crate) use event_placement::EventPlacement;
        pub(crate) use factory_return_visitor::FactoryReturnVisitor;
        pub(crate) use fake_placement::FakePlacement;
        pub(crate) use forbidden_import::ForbiddenImport;
        pub(crate) use forbidden_package_name::ForbiddenPackageName;
        pub(crate) use foreign_error_type::ForeignErrorType;
        pub(crate) use global_state::GlobalState;
        pub(crate) use global_state_visitor::GlobalStateVisitor;
        pub(crate) use impl_placement::ImplPlacement;
        pub(crate) use implementation_placement::ImplementationPlacement;
        pub(crate) use implementation_visibility::ImplementationVisibility;
        pub(crate) use import_through_file_module::ImportThroughFileModule;
        pub(crate) use in_memory_placement::InMemoryPlacement;
        pub(crate) use invariant_visitor::InvariantVisitor;
        pub(crate) use kind_content::KindContent;
        pub(crate) use kind_import_direction::KindImportDirection;
        pub(crate) use kind_shape::KindShape;
        pub(crate) use lifecycle_body_visitor::LifecycleBodyVisitor;
        pub(crate) use lifecycle_body_visitor::LifecycleFields;
        pub(crate) use listener_name::ListenerName;
        pub(crate) use main_shape::MainShape;
        pub(crate) use main_shape_visitor::MainShapeVisitor;
        pub(crate) use model_contains_functions::ModelContainsFunctions;
        pub(crate) use model_mutation_visitor::ModelMutationVisitor;
        pub(crate) use model_should_be_immutable::ModelShouldBeImmutable;
        pub(crate) use module_visibility::ModuleVisibility;
        pub(crate) use nested_data_struct::NestedDataStruct;
        pub(crate) use nested_enum::NestedEnum;
        pub(crate) use package_dependency_cycle::PackageDependencyCycle;
        pub(crate) use placement_type_visitor::PlacementTypeVisitor;
        pub(crate) use platform_attribute_visitor::PlatformAttributeVisitor;
        pub(crate) use platform_file_suffix::PlatformFileSuffix;
        pub(crate) use platform_import::PlatformImport;
        pub(crate) use platform_package::PlatformPackage;
        pub(crate) use plural_package_name::PluralPackageName;
        pub(crate) use re_export_shape::ReExportShape;
        pub(crate) use root_construction_visitor::RootConstructionVisitor;
        pub(crate) use root_file_shape::RootFileShape;
        pub(crate) use root_module_construction::RootModuleConstruction;
        pub(crate) use root_module_placement::RootModulePlacement;
        pub(crate) use system_clock_outside_di::SystemClockOutsideDi;
        pub(crate) use test_attribute_visitor::TestAttributeVisitor;
        pub(crate) use test_mirror::TestMirror;
        pub(crate) use unsafe_placement::UnsafePlacement;
        pub(crate) use unsafe_visitor::UnsafeVisitor;
        pub(crate) use use_case_abstraction::UseCaseAbstraction;
        pub(crate) use use_case_shape::UseCaseShape;
        pub(crate) use util_file_name::UtilFileName;
        pub(crate) use util_function_as_extension::UtilFunctionAsExtension;
        pub(crate) use util_shape::UtilShape;
        pub(crate) use visibility_tier::VisibilityTier;
        mod api_configuration;
        mod associated_factory_function;
        mod contract_placement;
        mod contract_unit_shape;
        mod core_shape;
        mod di_module_visibility;
        mod di_placement;
        mod doc_hidden_use;
        mod doc_hidden_visitor;
        mod elapsed_receiver_visitor;
        mod entry_point_shape;
        mod entry_point_visibility;
        mod enum_family_file;
        mod error_placement;
        mod event_placement;
        mod factory_return_visitor;
        mod fake_placement;
        mod forbidden_import;
        mod forbidden_package_name;
        mod foreign_error_type;
        mod impl_placement;
        mod implementation_placement;
        mod implementation_visibility;
        mod import_through_file_module;
        mod in_memory_placement;
        mod invariant_visitor;
        mod kind_content;
        mod kind_import_direction;
        mod kind_shape;
        mod listener_name;
        mod model_contains_functions;
        mod model_mutation_visitor;
        mod model_should_be_immutable;
        mod module_visibility;
        mod nested_data_struct;
        mod nested_enum;
        mod package_dependency_cycle;
        mod placement_type_visitor;
        mod platform_attribute_visitor;
        mod platform_file_suffix;
        mod platform_import;
        mod platform_package;
        mod plural_package_name;
        mod re_export_shape;
        mod root_file_shape;
        mod root_module_placement;
        mod test_attribute_visitor;
        mod test_mirror;
        mod unsafe_placement;
        mod unsafe_visitor;
        mod use_case_abstraction;
        mod use_case_shape;
        mod util_file_name;
        mod util_function_as_extension;
        mod util_shape;
        mod visibility_tier;

        mod constructor_dependency;
        mod di_module_lifecycle;
        mod di_module_shape;
        mod global_state;
        mod global_state_visitor;
        mod lifecycle_body_visitor;
        mod main_shape;
        mod main_shape_visitor;
        mod root_construction_visitor;
        mod root_module_construction;
        mod system_clock_outside_di;
    }

    pub(crate) mod di {
        pub(crate) use module_construction_rule_module::ModuleConstructionRuleModule;
        pub(crate) use module_ownership_rule_module::ModuleOwnershipRuleModule;
        pub(crate) use module_rule_module::ModuleRuleModule;
        pub(crate) use module_shape_rule_module::ModuleShapeRuleModule;
        pub(crate) use module_tree_rule_module::ModuleTreeRuleModule;
        mod module_construction_rule_module;
        mod module_ownership_rule_module;
        mod module_rule_module;
        mod module_shape_rule_module;
        mod module_tree_rule_module;
    }
}

pub(crate) mod expression {
    pub(crate) mod rule {
        pub(crate) use adapter_chain_visitor::AdapterChainVisitor;
        pub(crate) use boolean_operator_wrapping::BooleanOperatorWrapping;
        pub(crate) use closure_single_if_guard::ClosureSingleIfGuard;
        pub(crate) use expression_visitor::ExpressionVisitor;
        pub(crate) use operator_before_trailing_call::OperatorBeforeTrailingCall;
        pub(crate) use trivial_closure_path::TrivialClosurePath;
        mod adapter_chain_visitor;
        mod boolean_operator_wrapping;
        mod closure_single_if_guard;
        mod expression_visitor;
        mod operator_before_trailing_call;
        mod trivial_closure_path;
    }

    pub(crate) mod di {
        pub(crate) use expression_rule_module::ExpressionRuleModule;
        mod expression_rule_module;
    }
}

pub(crate) mod runtime {
    pub(crate) mod util {
        pub(crate) use format_placeholder::names_format_argument;
        pub(crate) use guard_type::guard_collection_fields;
        pub(crate) use lint_expectation::expects_lint;
        pub(crate) use node_predicate::node_predicate;
        pub(crate) use result_variant::is_result_variant;
        mod format_placeholder;
        mod guard_type;
        mod lint_expectation;
        mod node_predicate;
        mod result_variant;
    }
    pub(crate) mod rule {
        pub(crate) use await_visitor::AwaitVisitor;
        pub(crate) use block_on_in_test::BlockOnInTest;
        pub(crate) use block_on_outside_boundary::BlockOnOutsideBoundary;
        pub(crate) use blocking_call_in_async::BlockingCallInAsync;
        pub(crate) use blocking_lock_in_sync::BlockingLockInSync;
        pub(crate) use bridge_cleanup::BridgeCleanup;
        pub(crate) use broadcast_lag::BroadcastLag;
        pub(crate) use callback_bridge_shape::CallbackBridgeShape;
        pub(crate) use catch_unwind::CatchUnwind;
        pub(crate) use closure_effects::ClosureEffects;
        pub(crate) use critical_section_write::CriticalSectionWrite;
        pub(crate) use detached_task::DetachedTask;
        pub(crate) use duration_overload_call::DurationOverloadCall;
        pub(crate) use error_logging::ErrorLogging;
        pub(crate) use fallible_step_count::FallibleStepCount;
        pub(crate) use fallible_step_visitor::FallibleStepVisitor;
        pub(crate) use guard_writes::GuardWrites;
        pub(crate) use ignored_test::IgnoredTest;
        pub(crate) use impure_update_closure::ImpureUpdateClosure;
        pub(crate) use join_set_behind_lock::JoinSetBehindLock;
        pub(crate) use lock_choice::LockChoice;
        pub(crate) use model_rule_test::ModelRuleTest;
        pub(crate) use result_consumption::ResultConsumption;
        pub(crate) use runtime_visitor::RuntimeVisitor;
        pub(crate) use select_branches::SelectBranches;
        pub(crate) use select_cancel_safety::SelectCancelSafety;
        pub(crate) use task_visitor::TaskVisitor;
        pub(crate) use test_runtime_visitor::TestRuntimeVisitor;
        pub(crate) use test_time_paused::TestTimePaused;
        pub(crate) use thread_primitive::ThreadPrimitive;
        pub(crate) use time_primitive::TimePrimitive;
        pub(crate) use typed_runtime_visitor::TypedRuntimeVisitor;
        pub(crate) use watch_timeout::WatchTimeout;
        mod await_visitor;
        mod block_on_in_test;
        mod block_on_outside_boundary;
        mod blocking_call_in_async;
        mod blocking_lock_in_sync;
        mod bridge_cleanup;
        mod broadcast_lag;
        mod callback_bridge_shape;
        mod catch_unwind;
        mod closure_effects;
        mod critical_section_write;
        mod detached_task;
        mod duration_overload_call;
        mod error_logging;
        mod fallible_step_count;
        mod fallible_step_visitor;
        mod guard_writes;
        mod ignored_test;
        mod impure_update_closure;
        mod join_set_behind_lock;
        mod lock_choice;
        mod model_rule_test;
        mod result_consumption;
        mod runtime_visitor;
        mod select_branches;
        mod select_cancel_safety;
        mod task_visitor;
        mod test_runtime_visitor;
        mod test_time_paused;
        mod thread_primitive;
        mod time_primitive;
        mod typed_runtime_visitor;
        mod watch_timeout;
    }

    pub(crate) mod di {
        pub(crate) use runtime_rule_module::RuntimeRuleModule;
        mod runtime_rule_module;
    }
}

pub(crate) mod declaration {
    pub(crate) mod rule {
        pub(crate) use bare_identifier::BareIdentifier;
        pub(crate) use callback_declaration::CallbackDeclaration;
        pub(crate) use callback_visitor::CallbackVisitor;
        pub(crate) use contract_shape::ContractShape;
        pub(crate) use error_family_shape::ErrorFamilyShape;
        pub(crate) use initializer_on_declaration_line::InitializerOnDeclarationLine;
        pub(crate) use initializer_visitor::InitializerVisitor;
        pub(crate) use local_item::LocalItem;
        pub(crate) use model_derives::ModelDerives;
        pub(crate) use namespace_type::NamespaceType;
        pub(crate) use type_alias_on_one_line::TypeAliasOnOneLine;
        pub(crate) use type_alias_visitor::TypeAliasVisitor;
        pub(crate) use value_object_shape::ValueObjectShape;
        mod bare_identifier;
        mod callback_declaration;
        mod callback_visitor;
        mod contract_shape;
        mod error_family_shape;
        mod initializer_on_declaration_line;
        mod initializer_visitor;
        mod local_item;
        mod model_derives;
        mod namespace_type;
        mod type_alias_on_one_line;
        mod type_alias_visitor;
        mod value_object_shape;
    }
    pub(crate) mod util {
        pub(crate) use callback_field_type::callback_field_type;
        pub(crate) use contract_method_shape::invalid_contract_method;
        pub(crate) use eq_hash_shape::supports_eq_hash;
        pub(crate) use type_shape::{
            derives, has_bound, has_kind, has_standard_bound, is_technology,
        };
        mod callback_field_type;
        mod contract_method_shape;
        mod eq_hash_shape;
        mod type_shape;
    }

    pub(crate) mod di {
        pub(crate) use declaration_rule_module::DeclarationRuleModule;
        mod declaration_rule_module;
    }
}

pub(crate) mod readme {
    pub(crate) mod rule {
        pub(crate) use unit_readme::UnitReadme;
        mod unit_readme;
    }
    pub(crate) mod util {
        pub(crate) use sentence_count::count_sentences;
        mod sentence_count;
    }

    pub(crate) mod di {
        pub(crate) use readme_rule_module::ReadmeRuleModule;
        mod readme_rule_module;
    }
}

pub(crate) mod tuple {
    pub(crate) mod rule {
        pub(crate) use anonymous_tuple::AnonymousTuple;
        pub(crate) use tuple_binding_visitor::TupleBindingVisitor;
        pub(crate) use tuple_type_visitor::TupleTypeVisitor;
        mod anonymous_tuple;
        mod tuple_binding_visitor;
        mod tuple_type_visitor;
    }

    pub(crate) mod di {
        pub(crate) use tuple_rule_module::TupleRuleModule;
        mod tuple_rule_module;
    }
}

#[cfg(test)]
#[path = "../test"]
mod test {

    mod expression {
        mod rule {
            mod boolean_operator_wrapping_test;
            mod closure_single_if_guard_test;
            mod operator_before_trailing_call_test;
        }
    }
    mod runtime {
        mod rule {
            mod block_on_in_test_test;
            mod block_on_outside_boundary_test;
            mod blocking_call_in_async_test;
            mod blocking_lock_in_sync_test;
            mod bridge_cleanup_test;
            mod broadcast_lag_test;
            mod callback_bridge_shape_test;
            mod catch_unwind_test;
            mod critical_section_write_test;
            mod detached_task_test;
            mod duration_overload_call_test;
            mod error_logging_test;
            mod ignored_test_test;
            mod impure_update_closure_test;
            mod join_set_behind_lock_test;
            mod lock_choice_test;
            mod model_rule_test_test;
            mod result_consumption_test;
            mod select_cancel_safety_test;
            mod test_time_paused_test;
            mod thread_primitive_test;
            mod time_primitive_test;
            mod watch_timeout_test;
        }
    }
    pub(crate) mod declaration {
        pub(crate) mod util {
            pub(crate) use rule_case_test::{
                assert_code_findings, assert_code_fixture, read_code_context,
            };
            mod rule_case_test;
        }
        mod rule {
            mod bare_identifier_test;
            mod callback_declaration_test;
            mod contract_shape_test;
            mod error_family_shape_test;
            mod initializer_on_declaration_line_test;
            mod local_item_test;
            mod model_derives_test;
            mod namespace_type_test;
            mod phase4_declaration_docs_test;
            mod type_alias_on_one_line_test;
            mod value_object_shape_test;
        }
    }
    mod check {
        mod file {
            mod file_report_writer_test;
        }
        pub(crate) mod usecase {
            pub(crate) use check_workspace_use_case_test::read_case_context;
            mod check_workspace_use_case_test;
        }
        mod util {
            mod api_ownership_build_test;
            mod report_ext_test;
        }
        mod di {
            mod rule_module_test;
        }
    }
    mod config {
        mod rule {
            mod lint_config_test;
        }
        mod util {
            mod config_parse_test;
            mod required_configuration_render_test;
            mod role_resolve_test;
        }
    }
    mod layout {
        mod rule {
            mod container_sources_test;
            mod crate_targets_test;
            mod nested_unit_test;
            mod platform_source_set_unit_test;
            mod plugin_impl_unit_test;
            mod project_dir_matches_path_test;
            mod published_version_test;
            mod root_build_subprojects_test;
            mod root_package_path_test;
            mod root_sources_test;
            mod shell_automation_test;
            mod unit_path_test;
            mod xtask_alias_test;
        }
    }
    mod dependency {
        pub(crate) mod util {
            pub(crate) use role_dependency_test::read_role_context;
            mod role_dependency_test;
        }
        mod rule {
            mod api_unit_dependency_test;
            mod api_unit_platform_dependency_test;
            mod dependency_floor_test;
            mod entry_point_dependency_test;
            mod platform_dependency_table_test;
            mod technology_dependency_test;
            mod time_crate_test;
            mod unit_dependency_cycle_test;
            mod unit_dependency_test;
        }
    }
    mod readme {
        mod rule {
            mod unit_readme_test;
        }
        mod util {
            mod sentence_count_test;
        }
    }
    pub(crate) mod source {
        pub(crate) mod syn {
            pub(crate) use crate_index_builder_test::build_index;
            pub(crate) use crate_index_builder_test::build_index_files;
            mod crate_index_builder_test;
            mod expectation_visitor_test;
            mod module_tree_walker_test;
            mod syn_source_reader_test;
        }
        pub(crate) mod util {
            pub(crate) use workspace_name_index_test::read_resolution_context;
            mod cfg_predicate_ext_test;
            mod comment_scan_test;
            mod macro_arguments_test;
            mod syntax_record_test;
            mod target_predicate_test;
            mod workspace_name_index_test;
        }
    }
    mod module {
        pub(crate) mod util {
            pub(crate) use di_inspect_test::{assert_di_fixture, assert_di_lines, read_di_context};
            pub(crate) use import_case_test::assert_import_pair;
            pub(crate) use import_case_test::assert_import_workspace;
            pub(crate) use import_case_test::read_import_context;
            pub(crate) use placement_case_test::{
                assert_placement_findings, assert_placement_fixture, read_placement_context,
            };
            pub(crate) use resolver_case_test::assert_resolution_cases;
            pub(crate) use shape_inspect_test::{read_shape_context, read_workspace_shape_context};
            pub(crate) use workspace_provider_test::add_workspace_provider;
            mod di_inspect_test;
            mod import_case_test;
            mod placement_case_test;
            mod registration_base_test;
            mod resolver_case_test;
            mod shape_inspect_test;
            mod workspace_provider_test;
        }
        mod rule {
            mod api_configuration_test;
            mod associated_factory_function_test;
            mod constructor_dependency_test;
            mod contract_placement_test;
            mod contract_unit_shape_test;
            mod core_shape_test;
            mod di_module_lifecycle_test;
            mod di_module_shape_test;
            mod di_placement_test;
            mod entry_point_shape_test;
            mod entry_point_visibility_test;
            mod enum_family_file_test;
            mod error_placement_test;
            mod event_placement_test;
            mod fake_placement_test;
            mod forbidden_import_test;
            mod forbidden_package_name_test;
            mod foreign_error_type_test;
            mod global_state_test;
            mod impl_placement_test;
            mod implementation_placement_test;
            mod implementation_visibility_test;
            mod import_through_file_module_test;
            mod in_memory_placement_test;
            mod kind_content_test;
            mod kind_import_direction_test;
            mod kind_shape_test;
            mod listener_name_test;
            mod main_shape_test;
            mod model_contains_functions_test;
            mod model_mutation_visitor_test;
            mod model_should_be_immutable_test;
            mod module_visibility_test;
            mod package_dependency_cycle_test;
            mod platform_file_suffix_test;
            mod platform_import_test;
            mod platform_package_test;
            mod plural_package_name_test;
            mod re_export_shape_test;
            mod root_file_shape_test;
            mod root_module_construction_test;
            mod root_module_placement_test;
            mod system_clock_outside_di_test;
            mod test_mirror_test;
            mod unsafe_placement_test;
            mod use_case_abstraction_test;
            mod use_case_shape_test;
            mod util_file_name_test;
            mod util_function_as_extension_test;
            mod util_shape_test;
            mod visibility_tier_test;
            mod workspace_visibility_test;
        }
    }
    mod naming {
        mod rule {
            mod closure_parameter_name_test;
            mod explicit_it_name_test;
            mod function_verb_test;
            mod implementation_name_test;
            mod kind_name_in_package_test;
            mod roleless_type_name_test;
            mod test_function_name_test;
        }
    }
    mod ordering {
        mod rule {
            mod callee_before_caller_test;
            mod declaration_order_test;
        }
    }
    mod family {
        mod rule {
            mod constant_placement_test;
            mod one_type_family_per_file_test;
        }
    }
    mod default {
        mod rule {
            mod struct_default_value_test;
        }
    }
    mod import {
        mod rule {
            mod import_ordering_test;
        }
    }
    mod suppression {
        pub(crate) mod util {
            pub(crate) use suppression_apply_test::build_workspace;
            mod suppression_apply_test;
        }
        mod rule {
            mod expect_scope_test;
        }
    }
    mod syntax {
        pub(crate) mod util {
            pub(crate) use item_shape_test::assert_syntax_findings;
            mod item_shape_test;
        }
    }
    mod tuple {
        mod rule {
            mod tuple_binding_visitor_test;
            mod tuple_type_visitor_test;
        }
    }
    mod workspace {
        mod cargo {
            mod cargo_rustfmt_reader_test;
            mod cargo_workspace_reader_test;
        }
        pub(crate) mod util {
            pub(crate) use dependency_identity_test::read_dependency_metadata;
            mod dependency_identity_test;
            mod lock_dependency_test;
            mod toml_file_parse_test;
            mod toml_value_ext_test;
            mod toolchain_path_test;
        }
    }
    mod comment {
        mod rule {
            mod doc_comment_visitor_test;
            mod doc_scope_visitor_test;
            mod forbidden_comment_test;
            mod intra_doc_link_test;
            mod rustdoc_filler_test;
            mod rustdoc_html_tag_test;
        }
    }
}
