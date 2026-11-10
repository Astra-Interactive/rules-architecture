use crate::check::api::Rule;
use crate::check::model::Anchor;
use crate::check::model::Finding;
use crate::check::model::Location;
use crate::check::model::RuleContext;
use crate::check::model::RuleFinding;
use crate::check::model::RuleId;
use crate::config::model::CarrierLint;
use crate::config::model::RequiredConfiguration;
use crate::workspace::model::RustfmtFacts;
use crate::workspace::model::TomlFile;
use crate::workspace::model::TomlKind;
use crate::workspace::model::TomlValue;
use crate::workspace::model::ToolchainFile;
use crate::workspace::model::Workspace;
use crate::workspace::util::parse_toml_file;

pub(crate) struct LintConfig {
    required: RequiredConfiguration,
}

impl LintConfig {
    pub(crate) fn new(required: RequiredConfiguration) -> Self {
        Self { required }
    }

    fn build_finding(
        path: &str,
        line: Option<u32>,
        subject: String,
        message: String,
    ) -> RuleFinding {
        RuleFinding {
            finding: Finding {
                rule_id: RuleId(Self::RULE.to_owned()),
                location: Location {
                    path: path.to_owned(),
                    line,
                },
                subject,
                message,
            },
            anchor: Anchor::Unexpectable,
        }
    }

    fn find_level(value: &TomlValue) -> Option<&str> {
        value
            .as_str()
            .or_else(|| value.find(&["level"]).and_then(TomlValue::as_str))
    }

    fn find_priority(value: Option<&TomlValue>) -> i64 {
        value
            .and_then(|value| value.find(&["priority"]))
            .and_then(|priority| match &priority.kind {
                TomlKind::Integer(text) => text.parse().ok(),
                _ => None,
            })
            .unwrap_or(0)
    }

    fn is_lowered(level: Option<&str>) -> bool {
        matches!(level, Some("allow" | "expect"))
    }

    fn check_check_cfg(manifest: &TomlFile, own: &TomlValue, findings: &mut Vec<RuleFinding>) {
        let has_value = own
            .find(&["check-cfg"])
            .and_then(TomlValue::as_array)
            .unwrap_or_default()
            .iter()
            .any(|value| value.as_str() == Some("cfg(klibs_rules)"));
        if has_value {
            return;
        }
        let subject = "[workspace.lints.rust] unexpected_cfgs check-cfg".to_owned();
        let message = "check-cfg must contain \"cfg(klibs_rules)\", or the exception attributes stop the build".to_owned();
        findings.push(Self::build_finding(
            &manifest.path,
            Some(own.line),
            subject,
            message,
        ));
    }

    fn is_carrier_group(table: &str, group: &str) -> bool {
        Self::CARRIERS
            .iter()
            .any(|carrier| carrier.group == table && carrier.groups.contains(&group))
    }

    fn check_lowering_groups(
        manifest: &TomlFile,
        table_name: &str,
        required: &TomlValue,
        findings: &mut Vec<RuleFinding>,
    ) {
        let Some(table) = manifest.root.find(&["workspace", "lints", table_name]) else {
            return;
        };
        let lowest_priority = required
            .entries()
            .iter()
            .map(|lint| Self::find_priority(table.find(&[lint.key.as_str()])))
            .min()
            .unwrap_or(0);
        for entry in table.entries() {
            let is_group = Self::LINT_GROUPS.contains(&entry.key.as_str());
            let lowers = is_group
                && !Self::is_carrier_group(table_name, &entry.key)
                && Self::is_lowered(Self::find_level(&entry.value))
                && Self::find_priority(Some(&entry.value)) > lowest_priority;
            if lowers {
                let message = format!("the group `{}` lowers required lints", entry.key);
                findings.push(Self::build_finding(
                    &manifest.path,
                    Some(entry.value.line),
                    format!("[workspace.lints.{table_name}] {}", entry.key),
                    message,
                ));
            }
        }
    }

    fn check_required_lints(workspace: &Workspace, text: &str, findings: &mut Vec<RuleFinding>) {
        let manifest = &workspace.root_manifest;
        let required = match parse_toml_file("required workspace lints", text) {
            Ok(required) => required,
            Err(error) => {
                findings.push(Self::build_finding(
                    "required configuration",
                    None,
                    "required workspace lints".to_owned(),
                    error.to_string(),
                ));
                return;
            }
        };
        for group in required
            .root
            .find(&["workspace", "lints"])
            .map(TomlValue::entries)
            .unwrap_or_default()
        {
            let table_path = ["workspace", "lints", group.key.as_str()];
            let table = manifest.root.find(&table_path);
            for lint in group.value.entries() {
                let subject = format!("[workspace.lints.{}] {}", group.key, lint.key);
                let own = table.and_then(|table| table.find(&[lint.key.as_str()]));
                let Some(own) = own else {
                    let line = table.map(|table| table.line);
                    let message = "missing: the required configuration sets it to warn".to_owned();
                    findings.push(Self::build_finding(&manifest.path, line, subject, message));
                    continue;
                };
                let level = Self::find_level(own);
                if Self::is_lowered(level) || level.is_none() {
                    let message = format!(
                        "lowered to {}: it is required at warn, deny or forbid",
                        level.unwrap_or("no level")
                    );
                    findings.push(Self::build_finding(
                        &manifest.path,
                        Some(own.line),
                        subject,
                        message,
                    ));
                }
                if lint.key == "unexpected_cfgs" {
                    Self::check_check_cfg(manifest, own, findings);
                }
            }
            Self::check_lowering_groups(manifest, &group.key, &group.value, findings);
        }
    }

    fn check_carrier_lints(workspace: &Workspace, findings: &mut Vec<RuleFinding>) {
        let manifest = &workspace.root_manifest;
        for carrier in Self::CARRIERS {
            let table = manifest.root.find(&["workspace", "lints", carrier.group]);
            let own = table.and_then(|table| table.find(&[carrier.lint]));
            let subject = format!("[workspace.lints.{}] {}", carrier.group, carrier.lint);
            if let Some(own) = own.filter(|own| Self::is_lowered(Self::find_level(own))) {
                let message = "a default-on lint the documents rely on is lowered".to_owned();
                findings.push(Self::build_finding(
                    &manifest.path,
                    Some(own.line),
                    subject.clone(),
                    message,
                ));
            }
            let own_priority = Self::find_priority(own);
            for group_name in carrier.groups {
                let Some(group) = table.and_then(|table| table.find(&[*group_name])) else {
                    continue;
                };
                let lowers = Self::is_lowered(Self::find_level(group))
                    && Self::find_priority(Some(group)) >= own_priority;
                if lowers {
                    let message = format!("the group `{group_name}` lowers it");
                    findings.push(Self::build_finding(
                        &manifest.path,
                        Some(group.line),
                        subject.clone(),
                        message,
                    ));
                }
            }
        }
    }

    fn check_clippy_location(
        workspace: &Workspace,
        findings: &mut Vec<RuleFinding>,
    ) -> Option<TomlFile> {
        let root_files: Vec<&TomlFile> = workspace
            .clippy_files
            .iter()
            .filter(|file| !file.path.contains('/'))
            .collect();
        for file in workspace
            .clippy_files
            .iter()
            .filter(|file| file.path.contains('/'))
        {
            let message = "clippy reads this file instead of the workspace root's: keep one clippy.toml at the root".to_owned();
            findings.push(Self::build_finding(
                &file.path,
                None,
                file.path.clone(),
                message,
            ));
        }
        match root_files.as_slice() {
            [] => {
                let message =
                    "the required clippy.toml is missing at the workspace root".to_owned();
                findings.push(Self::build_finding(
                    "clippy.toml",
                    None,
                    "clippy.toml".to_owned(),
                    message,
                ));
                None
            }
            [file] => Some((*file).clone()),
            [first, ..] => {
                let message =
                    "both clippy.toml and .clippy.toml sit at the root: keep one".to_owned();
                findings.push(Self::build_finding(
                    &first.path,
                    None,
                    first.path.clone(),
                    message,
                ));
                Some((*first).clone())
            }
        }
    }

    fn is_zero(value: &TomlValue) -> bool {
        matches!(&value.kind, TomlKind::Integer(text) if text.parse::<i64>().ok() == Some(0))
    }

    fn check_clippy_thresholds(file: &TomlFile, findings: &mut Vec<RuleFinding>) {
        for key in ["too-many-lines-threshold", "excessive-nesting-threshold"] {
            let Some(value) = file.root.find(&[key]) else {
                let message =
                    "required: the clippy default differs from the documents' threshold".to_owned();
                findings.push(Self::build_finding(
                    &file.path,
                    None,
                    format!("{} {key}", file.path),
                    message,
                ));
                continue;
            };
            if key == "excessive-nesting-threshold" && Self::is_zero(value) {
                let message = "0 switches excessive_nesting off: it is required at 5".to_owned();
                findings.push(Self::build_finding(
                    &file.path,
                    Some(value.line),
                    format!("{} {key}", file.path),
                    message,
                ));
            }
        }
    }

    fn check_clippy_scalars(file: &TomlFile, findings: &mut Vec<RuleFinding>) {
        for key in [
            "allow-unwrap-in-tests",
            "allow-expect-in-tests",
            "check-private-items",
        ] {
            let is_true = file.root.find(&[key]).and_then(TomlValue::as_bool) == Some(true);
            if !is_true {
                let message = "required: true".to_owned();
                findings.push(Self::build_finding(
                    &file.path,
                    None,
                    format!("{} {key}", file.path),
                    message,
                ));
            }
        }
        Self::check_clippy_thresholds(file, findings);
        for key in ["allow-print-in-tests", "allow-dbg-in-tests"] {
            let Some(value) = file
                .root
                .find(&[key])
                .filter(|value| value.as_bool() == Some(true))
            else {
                continue;
            };
            let message = "no-print.rs.md bans console output in tests too".to_owned();
            findings.push(Self::build_finding(
                &file.path,
                Some(value.line),
                format!("{} {key}", file.path),
                message,
            ));
        }
    }

    fn check_clippy_path(
        file: &TomlFile,
        list: &str,
        required: &TomlValue,
        findings: &mut Vec<RuleFinding>,
    ) {
        let Some(path) = required.find(&["path"]).and_then(TomlValue::as_str) else {
            return;
        };
        let present = file
            .root
            .find(&[list])
            .and_then(TomlValue::as_array)
            .unwrap_or_default()
            .iter()
            .find(|value| {
                value
                    .as_str()
                    .or_else(|| value.find(&["path"]).and_then(TomlValue::as_str))
                    == Some(path)
            });
        let subject = format!("{} {list} {path}", file.path);
        let Some(present) = present else {
            let message = "a required path is missing".to_owned();
            findings.push(Self::build_finding(&file.path, None, subject, message));
            return;
        };
        let requires_allow_invalid = required
            .find(&["allow-invalid"])
            .and_then(TomlValue::as_bool)
            == Some(true);
        let allows_invalid = present
            .find(&["allow-invalid"])
            .and_then(TomlValue::as_bool)
            == Some(true);
        if requires_allow_invalid && !allows_invalid {
            findings.push(Self::build_finding(
                &file.path,
                Some(present.line),
                subject.clone(),
                "this required path must set allow-invalid = true".to_owned(),
            ));
        }
        let lacks_reason = list == "disallowed-types"
            && Self::REASON_PATHS.contains(&path)
            && present
                .find(&["reason"])
                .and_then(TomlValue::as_str)
                .is_none_or(|reason| reason.trim().is_empty());
        if lacks_reason {
            findings.push(Self::build_finding(
                &file.path,
                Some(present.line),
                subject,
                "this required path must carry a non-blank reason".to_owned(),
            ));
        }
    }

    fn check_clippy_paths(file: &TomlFile, text: &str, findings: &mut Vec<RuleFinding>) {
        let required = match parse_toml_file("required clippy.toml", text) {
            Ok(required) => required,
            Err(error) => {
                findings.push(Self::build_finding(
                    "required configuration",
                    None,
                    "required clippy.toml".to_owned(),
                    error.to_string(),
                ));
                return;
            }
        };
        for list in required
            .root
            .entries()
            .iter()
            .filter(|entry| entry.value.as_array().is_some())
        {
            let required_paths = list.value.as_array().unwrap_or_default();
            for required in required_paths {
                Self::check_clippy_path(file, &list.key, required, findings);
            }
        }
    }

    fn check_members(workspace: &Workspace, findings: &mut Vec<RuleFinding>) {
        for member in &workspace.members {
            let inherits = member
                .manifest
                .root
                .find(&["lints", "workspace"])
                .and_then(TomlValue::as_bool)
                == Some(true);
            if inherits {
                continue;
            }
            let message =
                "every member takes the workspace lints: `[lints] workspace = true`".to_owned();
            findings.push(Self::build_finding(
                &member.manifest.path,
                None,
                member.name.clone(),
                message,
            ));
        }
    }

    fn check_resolver(workspace: &Workspace, findings: &mut Vec<RuleFinding>) {
        let resolver = workspace
            .root_manifest
            .root
            .find(&["workspace", "resolver"]);
        if resolver.and_then(TomlValue::as_str) == Some("3") {
            return;
        }
        let message =
            "`resolver = \"3\"` keeps dependency versions within the project's Rust version"
                .to_owned();
        let line = resolver.map(|value| value.line);
        findings.push(Self::build_finding(
            &workspace.root_manifest.path,
            line,
            "[workspace] resolver".to_owned(),
            message,
        ));
    }

    fn is_dated_stable(channel: &str) -> bool {
        let Some(date) = channel.strip_prefix("stable-") else {
            return false;
        };
        let widths: Vec<usize> = date.split('-').map(str::len).collect();
        widths == [4, 2, 2]
            && date
                .chars()
                .all(|character| character == '-' || character.is_ascii_digit())
    }

    fn is_supported_channel(channel: &str) -> bool {
        if channel == "stable" || Self::is_dated_stable(channel) {
            return true;
        }
        let parts: Vec<u32> = channel
            .split('.')
            .map_while(|part| part.parse().ok())
            .collect();
        let is_version = !parts.is_empty() && parts.len() == channel.split('.').count();
        let major = parts.first().copied().unwrap_or(0);
        let minor = parts.get(1).copied().unwrap_or(0);
        is_version && (major > 1 || (major == 1 && minor >= Self::FLOOR_MINOR))
    }

    fn check_toolchain(workspace: &Workspace, findings: &mut Vec<RuleFinding>) {
        let (path, channel, line) = match &workspace.toolchain_file {
            None => return,
            Some(ToolchainFile::Legacy { path, channel }) => (path.clone(), channel.clone(), None),
            Some(ToolchainFile::Toml(file)) => {
                let Some(value) = file.root.find(&["toolchain", "channel"]) else {
                    return;
                };
                (
                    file.path.clone(),
                    value.as_str().unwrap_or_default().to_owned(),
                    Some(value.line),
                )
            }
        };
        if Self::is_supported_channel(&channel) {
            return;
        }
        let message =
            format!("\"{channel}\" is not a stable toolchain at or above the floor 1.93.0");
        findings.push(Self::build_finding(
            &path,
            line,
            format!("{path} channel"),
            message,
        ));
    }

    fn check_rustfmt(workspace: &Workspace, facts: &RustfmtFacts, findings: &mut Vec<RuleFinding>) {
        let nightly_only_keys = match facts {
            RustfmtFacts::NotQueried => return,
            RustfmtFacts::Missing { binary } => {
                let message = format!("no rustfmt at {binary}: install the rustfmt component");
                let path = workspace
                    .rustfmt_files
                    .first()
                    .map_or("rustfmt.toml", |file| file.path.as_str());
                findings.push(Self::build_finding(
                    path,
                    None,
                    "rustfmt".to_owned(),
                    message,
                ));
                return;
            }
            RustfmtFacts::Queried { nightly_only_keys } => nightly_only_keys,
        };
        for file in &workspace.rustfmt_files {
            for entry in file
                .root
                .entries()
                .iter()
                .filter(|entry| nightly_only_keys.contains(&entry.key))
            {
                let message =
                    "nightly-only for this rustfmt: a stable rustfmt ignores it".to_owned();
                findings.push(Self::build_finding(
                    &file.path,
                    Some(entry.line),
                    format!("{} {}", file.path, entry.key),
                    message,
                ));
            }
        }
    }

    fn find_forbidden_flag(flags: &[String]) -> Option<String> {
        flags.iter().enumerate().find_map(|(index, flag)| {
            let next = flags.get(index + 1).map(String::as_str);
            let is_cfg =
                (flag == "--cfg" && next == Some("klibs_rules")) || flag == "--cfg=klibs_rules";
            let is_cap = flag.starts_with("--cap-lints");
            let is_allow = flag.starts_with("-A") || flag.starts_with("--allow");
            (is_cfg || is_cap || is_allow).then(|| flag.clone())
        })
    }

    fn find_rustflags_values(config: &TomlFile) -> Vec<&TomlValue> {
        let mut values: Vec<&TomlValue> = Vec::new();
        values.extend(config.root.find(&["build", "rustflags"]));
        values.extend(config.root.find(&["rustflags"]));
        if let Some(targets) = config.root.find(&["target"]) {
            values.extend(
                targets
                    .entries()
                    .iter()
                    .filter_map(|entry| entry.value.find(&["rustflags"])),
            );
        }
        values
    }

    fn check_rustc_bootstrap(config: &TomlFile, findings: &mut Vec<RuleFinding>) {
        let Some(entry) = config
            .root
            .find(&["env"])
            .and_then(|environment| environment.find_entry("RUSTC_BOOTSTRAP"))
        else {
            return;
        };
        let value = entry
            .value
            .as_str()
            .or_else(|| entry.value.find(&["value"]).and_then(TomlValue::as_str));
        if value != Some("-1") {
            findings.push(Self::build_finding(
                &config.path,
                Some(entry.line),
                format!("{} env RUSTC_BOOTSTRAP", config.path),
                "RUSTC_BOOTSTRAP lets the stable compiler use nightly features; set only -1"
                    .to_owned(),
            ));
        }
    }

    fn check_cargo_config(workspace: &Workspace, findings: &mut Vec<RuleFinding>) {
        let Some(config) = &workspace.cargo_config else {
            return;
        };
        for value in Self::find_rustflags_values(config) {
            let flags: Vec<String> = match &value.kind {
                TomlKind::String(text) => text.split_whitespace().map(ToOwned::to_owned).collect(),
                _ => value
                    .as_array()
                    .unwrap_or_default()
                    .iter()
                    .filter_map(|flag| flag.as_str().map(ToOwned::to_owned))
                    .collect(),
            };
            if let Some(flag) = Self::find_forbidden_flag(&flags) {
                let message = "rustflags may not set `--cfg klibs_rules` or lower lints".to_owned();
                findings.push(Self::build_finding(
                    &config.path,
                    Some(value.line),
                    format!("{} rustflags {flag}", config.path),
                    message,
                ));
            }
        }
        Self::check_rustc_bootstrap(config, findings);
        if let Some(unstable) = config.root.find_entry("unstable") {
            let message = "an [unstable] table needs a nightly toolchain".to_owned();
            findings.push(Self::build_finding(
                &config.path,
                Some(unstable.line),
                format!("{} unstable", config.path),
                message,
            ));
        }
    }

    fn check_cargo_features(workspace: &Workspace, findings: &mut Vec<RuleFinding>) {
        let manifests = std::iter::once(&workspace.root_manifest)
            .chain(workspace.members.iter().map(|member| &member.manifest));
        for manifest in manifests {
            if let Some(entry) = manifest.root.find_entry("cargo-features") {
                let message = "`cargo-features` needs a nightly toolchain".to_owned();
                findings.push(Self::build_finding(
                    &manifest.path,
                    Some(entry.line),
                    format!("{} cargo-features", manifest.path),
                    message,
                ));
            }
        }
    }

    const RULE: &str = "LintConfig";
    const REASON_PATHS: [&str; 3] = [
        "std::sync::Mutex",
        "std::sync::RwLock",
        "std::sync::PoisonError",
    ];
    const FLOOR_MINOR: u32 = 93;
    const LINT_GROUPS: [&str; 15] = [
        "all",
        "correctness",
        "suspicious",
        "style",
        "complexity",
        "perf",
        "pedantic",
        "restriction",
        "nursery",
        "cargo",
        "unused",
        "nonstandard_style",
        "future_incompatible",
        "rust_2018_idioms",
        "warnings",
    ];
    const CARRIERS: [CarrierLint; 14] = [
        CarrierLint {
            group: "rust",
            lint: "unused_must_use",
            groups: &["unused", "warnings"],
        },
        CarrierLint {
            group: "rust",
            lint: "dead_code",
            groups: &["unused", "warnings"],
        },
        CarrierLint {
            group: "rust",
            lint: "unused_variables",
            groups: &["unused", "warnings"],
        },
        CarrierLint {
            group: "rust",
            lint: "non_snake_case",
            groups: &["nonstandard_style", "warnings"],
        },
        CarrierLint {
            group: "rust",
            lint: "unfulfilled_lint_expectations",
            groups: &["warnings"],
        },
        CarrierLint {
            group: "rust",
            lint: "deprecated",
            groups: &["warnings"],
        },
        CarrierLint {
            group: "clippy",
            lint: "disallowed_methods",
            groups: &["all", "style"],
        },
        CarrierLint {
            group: "clippy",
            lint: "disallowed_types",
            groups: &["all", "style"],
        },
        CarrierLint {
            group: "clippy",
            lint: "await_holding_lock",
            groups: &["all", "suspicious"],
        },
        CarrierLint {
            group: "clippy",
            lint: "await_holding_invalid_type",
            groups: &["all", "suspicious"],
        },
        CarrierLint {
            group: "clippy",
            lint: "excessive_nesting",
            groups: &["all", "complexity"],
        },
        CarrierLint {
            group: "clippy",
            lint: "missing_safety_doc",
            groups: &["all", "style"],
        },
        CarrierLint {
            group: "clippy",
            lint: "new_without_default",
            groups: &["all", "style"],
        },
        CarrierLint {
            group: "clippy",
            lint: "redundant_closure",
            groups: &["all", "style"],
        },
    ];
}

impl Rule for LintConfig {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        let workspace = &context.workspace;
        let mut findings = Vec::new();
        Self::check_required_lints(workspace, &self.required.workspace_lints, &mut findings);
        Self::check_carrier_lints(workspace, &mut findings);
        if let Some(clippy) = Self::check_clippy_location(workspace, &mut findings) {
            Self::check_clippy_scalars(&clippy, &mut findings);
            Self::check_clippy_paths(&clippy, &self.required.clippy_toml, &mut findings);
        }
        Self::check_members(workspace, &mut findings);
        Self::check_resolver(workspace, &mut findings);
        Self::check_toolchain(workspace, &mut findings);
        Self::check_rustfmt(workspace, &context.rustfmt, &mut findings);
        Self::check_cargo_config(workspace, &mut findings);
        Self::check_cargo_features(workspace, &mut findings);
        findings
    }
}
