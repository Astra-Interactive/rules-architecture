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
        pub use check_error::CheckError;
        pub use finding::Finding;
        pub use location::Location;
        pub use report::Report;
        pub(crate) use rule_context::RuleContext;
        pub(crate) use rule_finding::RuleFinding;
        pub use rule_id::RuleId;
        mod accepted_exception;
        mod anchor;
        mod check_error;
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
        pub(crate) use report_build::build_report;
        pub(crate) use rule_finding_build::build_node_finding;
        pub(crate) use rule_finding_build::format_module_path;
        pub(crate) use unanalysable_findings::build_unanalysable_findings;
        mod report_build;
        mod report_ext;
        mod rule_finding_build;
        mod unanalysable_findings;
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
        pub(crate) use comment::Comment;
        pub(crate) use comment::CommentKind;
        pub(crate) use crate_sources::CrateSources;
        pub(crate) use expectation::Expectation;
        pub(crate) use expectation::ExpectationPosition;
        pub(crate) use module_frame::ModuleFrame;
        pub(crate) use module_id::ModuleId;
        pub(crate) use node_span::NodeSpan;
        pub(crate) use source_file::SourceFile;
        pub(crate) use source_load_error::SourceLoadError;
        pub(crate) use source_module::DeclaredVisibility;
        pub(crate) use source_module::ModuleDeclaration;
        pub(crate) use source_module::SourceModule;
        pub(crate) use source_tree::SourceTree;
        pub(crate) use text_position::TextPosition;
        pub(crate) use unanalysable_source::UnanalysableSite;
        pub(crate) use unanalysable_source::UnanalysableSource;
        mod comment;
        mod crate_sources;
        mod expectation;
        mod module_frame;
        mod module_id;
        mod node_span;
        mod source_file;
        mod source_load_error;
        mod source_module;
        mod source_tree;
        mod text_position;
        mod unanalysable_source;
    }
    pub(crate) mod util {
        pub(crate) use comment_scan::scan_comments;
        pub(crate) use macro_arguments::parse_macro_arguments;
        pub(crate) use syntax_record::find_declared_visibility;
        pub(crate) use syntax_record::find_node_span;
        pub(crate) use syntax_record::find_path_attribute;
        pub(crate) use syntax_record::format_tokens;
        pub(crate) use syntax_record::has_conditional_path;
        pub(crate) use target_predicate::find_single_target_value;
        pub(crate) use target_predicate::list_manifest_target_values;
        pub(crate) use target_predicate::list_target_values;
        mod comment_scan;
        mod macro_arguments;
        mod node_span_ext;
        mod syntax_record;
        mod target_predicate;
    }
    pub(crate) mod syn {
        pub(crate) use expectation_parser::ExpectationParser;
        pub(crate) use expectation_visitor::ExpectationVisitor;
        pub(crate) use module_tree_walker::ModuleTreeWalker;
        pub(crate) use syn_source_reader::SynSourceReader;
        mod expectation_parser;
        mod expectation_visitor;
        mod module_tree_walker;
        mod syn_source_reader;
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
        pub(crate) use doc_comment_visitor::DocCommentVisitor;
        pub(crate) use forbidden_comment::ForbiddenComment;
        mod doc_comment_visitor;
        mod forbidden_comment;
    }
    pub(crate) mod util {
        pub(crate) use comment_shape::find_forbidden_shape;
        mod comment_shape;
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
}

pub(crate) mod naming {
    pub(crate) mod rule {
        pub(crate) use closure_parameter_name::ClosureParameterName;
        pub(crate) use explicit_it_name::ExplicitItName;
        pub(crate) use function_verb::FunctionVerb;
        pub(crate) use implementation_name::ImplementationName;
        pub(crate) use kind_name_in_package::KindNameInPackage;
        pub(crate) use name_visitor::NameVisitor;
        pub(crate) use test_function_name::TestFunctionName;
        mod closure_parameter_name;
        mod explicit_it_name;
        mod function_verb;
        mod implementation_name;
        mod kind_name_in_package;
        mod name_visitor;
        mod test_function_name;
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
}

pub(crate) mod default {
    pub(crate) mod rule {
        pub(crate) use struct_default_value::StructDefaultValue;
        mod struct_default_value;
    }
}

pub(crate) mod syntax {
    pub(crate) mod util {
        pub(crate) use item_shape::{
            find_self_name, find_type_name, is_constructor, to_snake_case,
        };
        mod item_shape;
    }
}

pub(crate) mod layout {
    pub(crate) mod rule {
        pub(crate) use container_sources::ContainerSources;
        pub(crate) use crate_targets::CrateTargets;
        pub(crate) use nested_unit::NestedUnit;
        pub(crate) use project_dir_matches_path::ProjectDirMatchesPath;
        pub(crate) use root_build_subprojects::RootBuildSubprojects;
        pub(crate) use root_sources::RootSources;
        pub(crate) use unit_path::UnitPath;
        mod container_sources;
        mod crate_targets;
        mod nested_unit;
        mod project_dir_matches_path;
        mod root_build_subprojects;
        mod root_sources;
        mod unit_path;
    }
}

pub(crate) mod dependency {
    pub(crate) mod rule {
        pub(crate) use entry_point_dependency::EntryPointDependency;
        pub(crate) use unit_dependency_cycle::UnitDependencyCycle;
        mod entry_point_dependency;
        mod unit_dependency_cycle;
    }
}

pub(crate) mod module {
    pub(crate) mod rule {
        pub(crate) use fake_placement::FakePlacement;
        pub(crate) use forbidden_package_name::ForbiddenPackageName;
        pub(crate) use module_visibility::ModuleVisibility;
        pub(crate) use platform_attribute_visitor::PlatformAttributeVisitor;
        pub(crate) use platform_package::PlatformPackage;
        pub(crate) use root_file_shape::RootFileShape;
        pub(crate) use test_attribute_visitor::TestAttributeVisitor;
        pub(crate) use test_mirror::TestMirror;
        pub(crate) use util_file_name::UtilFileName;
        mod fake_placement;
        mod forbidden_package_name;
        mod module_visibility;
        mod platform_attribute_visitor;
        mod platform_package;
        mod root_file_shape;
        mod test_attribute_visitor;
        mod test_mirror;
        mod util_file_name;
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
}

pub(crate) mod tuple {
    pub(crate) mod rule {
        pub(crate) use anonymous_tuple::AnonymousTuple;
        pub(crate) use tuple_type_visitor::TupleTypeVisitor;
        mod anonymous_tuple;
        mod tuple_type_visitor;
    }
}

#[cfg(test)]
#[path = "../test"]
mod test {
    mod check {
        mod file {
            mod file_report_writer_test;
        }
        pub(crate) mod usecase {
            pub(crate) use check_workspace_use_case_test::read_case_context;
            mod check_workspace_use_case_test;
        }
        mod util {
            mod report_ext_test;
        }
    }
    mod config {
        mod rule {
            mod lint_config_test;
        }
        mod util {
            mod config_parse_test;
            mod required_configuration_render_test;
        }
    }
    mod layout {
        mod rule {
            mod container_sources_test;
            mod crate_targets_test;
            mod nested_unit_test;
            mod project_dir_matches_path_test;
            mod root_build_subprojects_test;
            mod unit_path_test;
        }
    }
    mod dependency {
        mod rule {
            mod entry_point_dependency_test;
            mod unit_dependency_cycle_test;
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
    mod source {
        mod syn {
            mod expectation_visitor_test;
            mod module_tree_walker_test;
            mod syn_source_reader_test;
        }
        mod util {
            mod comment_scan_test;
            mod macro_arguments_test;
            mod syntax_record_test;
            mod target_predicate_test;
        }
    }
    mod module {
        mod rule {
            mod module_visibility_test;
            mod platform_package_test;
            mod root_file_shape_test;
            mod test_mirror_test;
        }
    }
    mod naming {
        mod rule {
            mod closure_parameter_name_test;
            mod explicit_it_name_test;
            mod function_verb_test;
            mod implementation_name_test;
            mod kind_name_in_package_test;
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
        mod util {
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
        }
    }
}
