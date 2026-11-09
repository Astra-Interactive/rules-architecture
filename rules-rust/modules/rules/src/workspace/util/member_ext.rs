use crate::workspace::model::Member;
use crate::workspace::model::Target;
use crate::workspace::model::{TomlValue, Workspace};

impl Member {
    pub(crate) fn is_published(&self, workspace: &Workspace) -> bool {
        let version = self.manifest.root.find(&["package", "version"]);
        let version = if version
            .and_then(|value| value.find(&["workspace"]))
            .and_then(TomlValue::as_bool)
            == Some(true)
        {
            workspace
                .root_manifest
                .root
                .find(&["workspace", "package", "version"])
        } else {
            version
        };
        if version.and_then(TomlValue::as_str).is_none() {
            return false;
        }
        let publish = self.manifest.root.find(&["package", "publish"]);
        let publish = if publish
            .and_then(|value| value.find(&["workspace"]))
            .and_then(TomlValue::as_bool)
            == Some(true)
        {
            workspace
                .root_manifest
                .root
                .find(&["workspace", "package", "publish"])
        } else {
            publish
        };
        !publish.is_some_and(|value| {
            value.as_bool() == Some(false) || value.as_array().is_some_and(<[_]>::is_empty)
        })
    }

    pub(crate) fn find_lib_target(&self) -> Option<&Target> {
        self.targets.iter().find(|target| target.is_lib_target())
    }

    pub(crate) fn has_rust_library(&self) -> bool {
        self.targets.iter().any(Target::is_rust_library)
    }

    pub(crate) fn is_proc_macro(&self) -> bool {
        self.find_lib_target().is_some_and(Target::is_proc_macro)
    }
}
