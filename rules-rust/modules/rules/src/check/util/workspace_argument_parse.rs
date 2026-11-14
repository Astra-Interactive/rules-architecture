use crate::check::model::WorkspaceArgumentError;
use std::ffi::OsString;
use std::path::Path;
use std::path::PathBuf;

fn resolve_manifest_directory(manifest_path: &Path) -> Result<PathBuf, WorkspaceArgumentError> {
    match manifest_path.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => Ok(parent.to_path_buf()),
        Some(_) => Ok(PathBuf::from(".")),
        None => Err(WorkspaceArgumentError::Usage),
    }
}

pub(crate) fn parse_workspace_directory(
    arguments: &[OsString],
) -> Result<PathBuf, WorkspaceArgumentError> {
    let remaining = match arguments.split_first() {
        Some((first, rest)) if first == "klibs-rules" => rest,
        _ => arguments,
    };
    match remaining {
        [] => Ok(PathBuf::from(".")),
        [flag, manifest_path] if flag == "--manifest-path" => {
            resolve_manifest_directory(Path::new(manifest_path))
        }
        _ => Err(WorkspaceArgumentError::Usage),
    }
}
