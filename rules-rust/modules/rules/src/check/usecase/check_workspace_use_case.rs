use crate::check::api::Rule;
use crate::check::model::CheckError;
use crate::check::model::Report;
use crate::check::model::RuleContext;
use crate::check::model::RuleFinding;
use crate::check::util::build_report;
use crate::check::util::build_unanalysable_findings;
use crate::config::util::parse_project_config;
use crate::source::api::SourceReader;
use crate::source::model::Expectation;
use crate::source::syn::CrateIndexBuilder;
use crate::suppression::model::RuleActivity;
use crate::suppression::util::apply_suppressions;
use crate::suppression::util::validate_expectations;
use crate::workspace::api::RustfmtReader;
use crate::workspace::api::WorkspaceReader;
use crate::workspace::model::RustfmtFacts;
use std::path::Path;
use std::sync::Arc;

pub struct CheckWorkspaceUseCase {
    workspace_reader: Arc<dyn WorkspaceReader>,
    source_reader: Arc<dyn SourceReader>,
    rustfmt_reader: Arc<dyn RustfmtReader>,
    rules: Vec<Arc<dyn Rule>>,
    active_rule_ids: Vec<&'static str>,
}

impl CheckWorkspaceUseCase {
    fn read_context(&self, dir: &Path) -> Result<RuleContext, CheckError> {
        let workspace = self.workspace_reader.read_workspace(dir)?;
        let project_config = parse_project_config(&workspace)?;
        let crates = self.source_reader.read_sources(&workspace);
        let indexes = CrateIndexBuilder::build_all(&crates, &workspace);
        let rustfmt = if workspace.rustfmt_files.is_empty() {
            RustfmtFacts::NotQueried
        } else {
            self.rustfmt_reader.read_rustfmt_facts()
        };
        Ok(RuleContext {
            workspace,
            project_config,
            crates,
            indexes,
            names: std::sync::OnceLock::new(),
            rustfmt,
        })
    }

    pub(crate) fn new(
        workspace_reader: Arc<dyn WorkspaceReader>,
        source_reader: Arc<dyn SourceReader>,
        rustfmt_reader: Arc<dyn RustfmtReader>,
        rules: Vec<Arc<dyn Rule>>,
        active_rule_ids: Vec<&'static str>,
    ) -> Self {
        Self {
            workspace_reader,
            source_reader,
            rustfmt_reader,
            rules,
            active_rule_ids,
        }
    }

    /// `dir` is any directory inside the workspace; the root comes from `cargo metadata`. No file is written.
    ///
    /// # Errors
    ///
    /// Returns [`CheckError::Metadata`] when `cargo metadata` fails, [`CheckError::Configuration`] for an
    /// invalid workspace configuration or unresolved lockfile dependency selection, and [`CheckError::Io`]
    /// when a manifest or a configuration file cannot be read.
    pub fn invoke(&self, dir: &Path) -> Result<Report, CheckError> {
        self.invoke_with_ownership(dir)
            .map(|checked| checked.report)
    }

    /// Ownership advice uses the same parsed workspace as the violations and performs no writes.
    ///
    /// # Errors
    ///
    /// Returns the same metadata, configuration and I/O errors as [`Self::invoke`].
    pub fn invoke_with_ownership(
        &self,
        dir: &Path,
    ) -> Result<crate::check::model::CheckedWorkspace, CheckError> {
        let context = self.read_context(dir)?;
        let mut findings: Vec<RuleFinding> = self
            .rules
            .iter()
            .flat_map(|rule| rule.check(&context))
            .collect();
        findings.extend(build_unanalysable_findings(&context.crates));
        for index in &context.indexes {
            for unresolved in &index.unanalysable {
                if let crate::source::model::UnanalysableSite::Node { file, span } =
                    &unresolved.site
                {
                    findings.push(crate::check::util::build_node_finding(
                        "Unanalysable",
                        file,
                        *span,
                        unresolved.subject.clone(),
                        unresolved.message.clone(),
                    ));
                }
            }
        }
        let expectations: Vec<Expectation> = context
            .crates
            .iter()
            .flat_map(|sources| sources.expectations.iter().cloned())
            .collect();
        let activity = RuleActivity {
            active_rules: self
                .active_rule_ids
                .iter()
                .map(|id| (*id).to_owned())
                .chain(["Unanalysable".to_owned()])
                .collect(),
            is_platform_words_empty: context.project_config.platform_words.is_empty(),
        };
        let validated = validate_expectations(&expectations);
        let outcome = apply_suppressions(findings, validated, &context.project_config, &activity);
        let api_ownership = crate::check::util::build_api_ownership(&context);
        let report = build_report(
            outcome.violations,
            outcome.accepted,
            context.workspace.target_directory.clone(),
        );
        Ok(crate::check::model::CheckedWorkspace {
            report,
            api_ownership,
        })
    }
}
