use crate::source::model::ExternalPath;

impl ExternalPath {
    pub(crate) fn format_path(&self) -> String {
        std::iter::once(self.package.as_str())
            .chain(self.segments.iter().skip(1).map(String::as_str))
            .collect::<Vec<_>>()
            .join("::")
    }
}
