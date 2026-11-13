use crate::check::model::RuleContext;
use crate::source::model::Resolution;
use crate::source::model::WorkspaceResolvedPath;

impl RuleContext {
    fn has_library_name(member: &crate::workspace::model::Member, name: &str) -> bool {
        member.targets.iter().any(|target| {
            target.name.replace('-', "_") == name
                && target
                    .kinds
                    .iter()
                    .any(|kind| kind == "lib" || kind == "rlib")
        })
    }

    pub(crate) fn resolve_workspace_path(
        &self,
        index: usize,
        module: usize,
        path: &syn::Path,
    ) -> Vec<WorkspaceResolvedPath> {
        match self.indexes[index].resolve_path(module, path) {
            Resolution::Internal(paths) => paths
                .into_iter()
                .map(|path| WorkspaceResolvedPath { index, path })
                .collect(),
            Resolution::External(path) => self.resolve_dependency_path(index, &path, 0),
            _ => Vec::new(),
        }
    }

    fn resolve_dependency_path(
        &self,
        index: usize,
        path: &str,
        depth: usize,
    ) -> Vec<WorkspaceResolvedPath> {
        if depth > 8 {
            return Vec::new();
        }
        let mut segments: Vec<String> = path.split("::").map(str::to_owned).collect();
        if segments.is_empty() {
            return Vec::new();
        }
        let dependency_name = segments.remove(0);
        let owner = self
            .workspace
            .members
            .iter()
            .find(|member| member.name == self.indexes[index].crate_name);
        let dependency = owner.and_then(|owner| {
            owner
                .dependencies
                .iter()
                .find(|dependency| dependency.code_name == dependency_name)
        });
        let member = dependency
            .and_then(|dependency| dependency.path.as_ref())
            .and_then(|path| {
                self.workspace.members.iter().find(|member| {
                    member.directory == *path
                        || self.workspace.root.join(&member.directory) == std::path::Path::new(path)
                })
            })
            .or_else(|| owner.filter(|owner| Self::has_library_name(owner, &dependency_name)));
        let crate_name = member
            .map(|member| member.name.replace('-', "_"))
            .or_else(|| dependency.map(|dependency| dependency.name.replace('-', "_")))
            .unwrap_or(dependency_name);
        self.indexes
            .iter()
            .enumerate()
            .filter(|(_, index)| index.crate_name.replace('-', "_") == crate_name)
            .flat_map(
                |(ordinal, index)| match index.resolve_segments(0, &segments, 0) {
                    Resolution::Internal(paths) => paths
                        .into_iter()
                        .map(|path| WorkspaceResolvedPath {
                            index: ordinal,
                            path,
                        })
                        .collect(),
                    Resolution::External(path) => {
                        self.resolve_dependency_path(ordinal, &path, depth + 1)
                    }
                    _ => Vec::new(),
                },
            )
            .collect()
    }
}
