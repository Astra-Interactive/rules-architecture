use crate::source::api::SourceReader;
use crate::source::model::CrateSources;
use crate::source::model::ExpectationPosition;
use crate::source::model::ModuleId;
use crate::source::model::SourceTree;
use crate::source::syn::ExpectationVisitor;
use crate::source::syn::ModuleTreeWalker;
use crate::workspace::model::Member;
use crate::workspace::model::Target;
use crate::workspace::model::Workspace;
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

pub(crate) struct SynSourceReader;

impl SynSourceReader {
    fn retain_physical_node_expectations(crates: &mut [CrateSources]) {
        let mut seen = BTreeSet::new();
        for sources in crates {
            let files: BTreeSet<String> = sources
                .expectations
                .iter()
                .map(|expectation| expectation.file.clone())
                .collect();
            sources.expectations.retain(|expectation| {
                let is_physical_node = matches!(
                    expectation.position,
                    ExpectationPosition::Node(_) | ExpectationPosition::File
                );
                !is_physical_node || !seen.contains(&expectation.file)
            });
            seen.extend(files);
        }
    }

    fn has_tests_it(workspace: &Workspace, member: &Member) -> bool {
        member.is_proc_macro() || member.is_published(workspace)
    }

    fn is_tests_it_target(workspace: &Workspace, member: &Member, target: &Target) -> bool {
        let tests_it = if member.directory.is_empty() {
            "tests/it/main.rs".to_owned()
        } else {
            format!("{}/tests/it/main.rs", member.directory)
        };
        Self::has_tests_it(workspace, member)
            && target.kinds.iter().any(|kind| kind == "test")
            && target.src_path == tests_it
    }

    fn select_targets<'member>(
        workspace: &Workspace,
        member: &'member Member,
    ) -> Vec<&'member Target> {
        let mut seen = BTreeSet::new();
        member
            .targets
            .iter()
            .filter(|target| {
                target.is_lib_target()
                    || target.is_bin()
                    || Self::is_tests_it_target(workspace, member, target)
            })
            .filter(|target| seen.insert(target.src_path.clone()))
            .collect()
    }

    fn list_source_roots(workspace: &Workspace, member: &Member) -> Vec<String> {
        let mut roots: Vec<String> = ["src", "test", "fixture"]
            .iter()
            .map(|root| (*root).to_owned())
            .collect();
        if Self::has_tests_it(workspace, member) {
            roots.push("tests/it".to_owned());
        }
        roots
            .into_iter()
            .map(|root| {
                if member.directory.is_empty() {
                    root
                } else {
                    format!("{}/{root}", member.directory)
                }
            })
            .collect()
    }

    fn find_undeclared_files(
        member: &Member,
        workspace: &Workspace,
        crates: &[CrateSources],
    ) -> Vec<String> {
        let declared: BTreeSet<&str> = crates
            .iter()
            .flat_map(|sources| &sources.files)
            .map(|file| file.path.as_str())
            .collect();
        let roots = Self::list_source_roots(workspace, member);
        workspace
            .files
            .iter()
            .filter(|file| {
                let path = Path::new(file);
                let is_rust = path.extension().is_some_and(|extension| extension == "rs");
                let is_in_tree = roots
                    .iter()
                    .any(|root| file.starts_with(&format!("{root}/")));
                is_rust && is_in_tree && !declared.contains(file.as_str())
            })
            .cloned()
            .collect()
    }

    fn read_member_sources(workspace: &Workspace, member: &Member) -> Vec<CrateSources> {
        let mut crates: Vec<CrateSources> = Self::select_targets(workspace, member)
            .into_iter()
            .map(|target| {
                let tree = if Self::is_tests_it_target(workspace, member, target) {
                    SourceTree::TestsIt
                } else {
                    SourceTree::Src
                };
                ModuleTreeWalker::new(
                    workspace.root.clone(),
                    member.name.clone(),
                    target.name.clone(),
                )
                .walk_crate(&target.src_path, tree)
            })
            .collect();
        Self::retain_physical_node_expectations(&mut crates);
        let undeclared = Self::find_undeclared_files(member, workspace, &crates);
        for first in crates.iter_mut().take(1) {
            for file in &undeclared {
                let syntax = fs::read_to_string(workspace.root.join(file))
                    .ok()
                    .and_then(|text| syn::parse_file(&text).ok());
                let Some(syntax) = syntax else {
                    continue;
                };
                let module = ModuleId {
                    crate_name: first.crate_name.clone(),
                    target: first.target.clone(),
                    path: Vec::new(),
                };
                first.expectations.extend(
                    ExpectationVisitor::new(file.clone(), module).collect_file(&syntax, false),
                );
            }
            first.undeclared_files.clone_from(&undeclared);
        }
        crates
    }
}

impl SourceReader for SynSourceReader {
    fn read_sources(&self, workspace: &Workspace) -> Vec<CrateSources> {
        workspace
            .members
            .iter()
            .flat_map(|member| Self::read_member_sources(workspace, member))
            .collect()
    }
}
