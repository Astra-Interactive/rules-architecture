use crate::config::model::Role;
use crate::config::util::format_role_name;

impl Role {
    fn describe_role_crate(self) -> String {
        let name = format_role_name(self);
        let article = if name.starts_with(['a', 'e', 'i', 'o', 'u']) {
            "an"
        } else {
            "a"
        };
        format!("{article} {name} crate")
    }

    fn describe_row(self) -> &'static str {
        match self {
            Role::EntryPoint => "every crate except another entry-point crate",
            Role::Platform | Role::Context => {
                "api, core-api, core, core-platform and storage-contract crates"
            }
            Role::Api => "api, core-api, core and storage-contract crates",
            Role::Impl => "api, core-api and core crates",
            Role::ImplUi => "api, core-api, core and core-ui crates",
            Role::Technology => {
                "storage-contract, core-api and core crates, and the api crate beside it"
            }
            Role::StorageContract => "core-api and core crates",
            Role::CorePlatform | Role::CoreUi => "core-api crates",
            Role::ProcMacro | Role::Core | Role::CoreApi | Role::Contract => {
                "no crate of the project"
            }
        }
    }

    pub(crate) fn describe_requirement(self, to: Option<Role>, has_sibling_access: bool) -> String {
        match (self, to) {
            (Role::Technology, Some(Role::Api)) => {
                "a technology crate depends on the api crate beside it only".to_owned()
            }
            (Role::ImplUi, Some(Role::Impl)) if has_sibling_access => {
                "an impl-ui crate depends on an impl only when it is a sibling, in the same parent directory".to_owned()
            }
            (Role::ImplUi, Some(Role::Impl)) => {
                "an impl-ui crate depends on its sibling impl only with implementation-access = \"siblings\"".to_owned()
            }
            (Role::ImplUi, Some(Role::ImplUi)) => {
                "an impl-ui crate depends on another impl-ui only with composition = \"host\"".to_owned()
            }
            (_, Some(Role::Contract)) => {
                "a contract crate is reached from a route or a technology module".to_owned()
            }
            _ => format!("{} depends on {}", self.describe_role_crate(), self.describe_row()),
        }
    }
}
