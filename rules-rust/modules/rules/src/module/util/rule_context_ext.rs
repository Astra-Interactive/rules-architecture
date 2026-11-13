use crate::check::model::RuleContext;
use crate::workspace::model::TomlValue;

impl RuleContext {
    pub(crate) fn root_module_name(&self) -> &str {
        self.workspace
            .root_manifest
            .root
            .find(&["workspace", "metadata", "klibs-rules", "root-module"])
            .and_then(TomlValue::as_str)
            .unwrap_or("RootModule")
    }
}
