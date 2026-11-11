use crate::config::model::RequiredConfiguration;

const WORKSPACE_LINTS: &str = r#"[workspace.lints.rust]
unreachable_pub = "warn"
unexpected_cfgs = { level = "warn", check-cfg = ["cfg(klibs_rules)"] }

[workspace.lints.clippy]
mod_module_files = "warn"
self_named_module_files = "warn"
let_underscore_must_use = "warn"
unused_result_ok = "warn"
unwrap_used = "warn"
expect_used = "warn"
print_stdout = "warn"
print_stderr = "warn"
dbg_macro = "warn"
allow_attributes = "warn"
allow_attributes_without_reason = "warn"
too_many_lines = "warn"
undocumented_unsafe_blocks = "warn"
redundant_closure_for_method_calls = "warn"
"#;

const CLIPPY_TOML: &str = r#"allow-unwrap-in-tests = true
allow-expect-in-tests = true
too-many-lines-threshold = 60
excessive-nesting-threshold = 5
check-private-items = true
await-holding-invalid-types = [
  { path = "tokio::sync::watch::Ref", reason = "async.rs.md: let the Ref go before the next .await", allow-invalid = true },
]
disallowed-types = [
  { path = "std::sync::Mutex", reason = "async.rs.md, Shared state and locks: std locks poison" },
  { path = "std::sync::RwLock", reason = "async.rs.md, Shared state and locks: std locks poison" },
  { path = "std::sync::PoisonError", reason = "async.rs.md, Shared state and locks: std locks poison" },
]
disallowed-methods = [
  { path = "std::thread::spawn", reason = "no-callbacks.rs.md, Threads" },
  { path = "std::thread::Builder::spawn", reason = "no-callbacks.rs.md, Threads" },
  { path = "std::io::stdout", reason = "no-print.rs.md" },
  { path = "std::io::stderr", reason = "no-print.rs.md" },
  { path = "std::time::SystemTime::now", reason = "time-api.rs.md: the moment comes from the Clock" },
  { path = "std::time::SystemTime::elapsed", reason = "time-api.rs.md: the moment comes from the Clock" },
  { path = "chrono::Utc::now", reason = "time-api.rs.md: the moment comes from the Clock", allow-invalid = true },
  { path = "chrono::Local::now", reason = "time-api.rs.md: the moment comes from the Clock", allow-invalid = true },
  { path = "jiff::Timestamp::now", reason = "time-api.rs.md: the moment comes from the Clock", allow-invalid = true },
  { path = "jiff::Zoned::now", reason = "time-api.rs.md: the moment comes from the Clock", allow-invalid = true },
  { path = "time::OffsetDateTime::now_utc", reason = "time-api.rs.md: the moment comes from the Clock", allow-invalid = true },
  { path = "time::OffsetDateTime::now_local", reason = "time-api.rs.md: the moment comes from the Clock", allow-invalid = true },
  { path = "time::UtcDateTime::now", reason = "time-api.rs.md: the moment comes from the Clock", allow-invalid = true },
  { path = "tokio::sync::watch::Sender::send", reason = "async.rs.md: send_modify, send_if_modified or send_replace", allow-invalid = true },
]
"#;

pub fn render_required_configuration() -> RequiredConfiguration {
    RequiredConfiguration {
        workspace_lints: WORKSPACE_LINTS.to_owned(),
        clippy_toml: CLIPPY_TOML.to_owned(),
    }
}
