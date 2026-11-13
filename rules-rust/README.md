# rules-rust

`klibs-rules` checks a Cargo workspace against the Astra-Interactive architecture rules (the `.rs.md` documents of
`conf.ide/.claude/docs` and the language-neutral `arch-*.md`). It is the Rust counterpart of `rules-kotlin`: a
required rustc, clippy and rustfmt configuration plus rules of its own, run as a test of the consumer's `xtask`.

## Installation

1. `klibs-rules = "=X.Y.Z"` in the `[dependencies]` of the consumer's `xtask` crate.
2. The required configuration: the `[workspace.lints]` block and `clippy.toml` that
   `klibs_rules::config::util::render_required_configuration()` renders (below), `[lints] workspace = true` in every
   member, `resolver = "3"` in the root `[workspace]`.
3. `[workspace.metadata.klibs-rules] product-kind`, `platform-words` when the project has host frameworks, and the
   `role` of every crate the detection does not cover.
4. `[alias] xtask = "run --package xtask --"` in `.cargo/config.toml`.
5. The `xtask` template below: `xtask/Cargo.toml`, `xtask/README.md`, `xtask/src/main.rs`, the command and its test.
   A tooling crate is `src/main.rs` alone, so the template's `main.rs` holds the module tree and the test block; only
   an `entry-point` crate splits `src/lib.rs` (the tree) from `src/main.rs` (`use` lines and `fn main`).
6. Pin the toolchain in `rust-toolchain.toml`: `LintConfig` asks the rustfmt beside the running cargo which keys are
   nightly-only, and that list differs by release (`hex_literal_case` is stable on 1.99.0, nightly-only on 1.93.0), so
   an unpinned project can get a rustfmt finding on one machine and none on another.

```toml
# xtask/Cargo.toml (no `version`: the crate is not published)
[package]
name = "xtask"
edition = "2024"

[dependencies]
klibs-rules = "=0.2.0"

[lints]
workspace = true
```

```text
# xtask/README.md
Runs the build automation of this workspace, the architecture check among it.
```

```rust
// xtask/src/main.rs
use crate::architecture::command::run_architecture_check;
use std::env;
use std::path::Path;
use std::process::ExitCode;

pub(crate) mod architecture {
    pub(crate) mod command {
        pub(crate) use architecture_check_command::run_architecture_check;
        mod architecture_check_command;
    }
}

fn main() -> ExitCode {
    let task = env::args().nth(1);
    match task.as_deref() {
        Some("architecture-check") => run_architecture_check(Path::new(env!("CARGO_MANIFEST_DIR"))),
        _ => ExitCode::from(2),
    }
}

#[cfg(test)]
#[path = "../test"]
mod test {
    mod architecture {
        mod command {
            mod architecture_check_command_test;
        }
    }
}
```

```rust
// xtask/src/architecture/command/architecture_check_command.rs
use klibs_rules::check::di::CheckModule;
use std::path::Path;
use std::process::ExitCode;

#[expect(
    clippy::print_stdout,
    reason = "the architecture report is this command's product"
)]
pub(crate) fn run_architecture_check(workspace_dir: &Path) -> ExitCode {
    let check_module = CheckModule::new();
    let checked = match check_module.check_workspace_use_case.invoke_with_ownership(workspace_dir) {
        Ok(report) => report,
        Err(error) => {
            println!("{error}");
            return ExitCode::from(2);
        }
    };
    let report = &checked.report;
    println!("{}", report.to_violation_lines());
    if let Err(error) = check_module.report_writer.write_with_ownership(&checked) {
        println!("{error}");
        return ExitCode::from(2);
    }
    report.to_exit_code()
}
```

```rust
// xtask/test/architecture/command/architecture_check_command_test.rs
use klibs_rules::check::di::CheckModule;
use std::path::Path;

#[test]
fn given_this_workspace_when_checked_then_no_rule_is_violated() {
    let check_module = CheckModule::new();

    let report = check_module
        .check_workspace_use_case
        .invoke(Path::new(env!("CARGO_MANIFEST_DIR")))
        .expect("the klibs-rules configuration loads");

    assert!(report.is_clean(), "{}", report.to_violation_lines());
}
```

`cases/consumer/fixed` is this template in a workspace that passes; `cases/consumer/failing` reports two findings;
`cases/consumer/single-crate` is a root `[package]` before its re-layout (`RootSources`).

Locally and in CI: `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`
(runs the architecture test). `cargo xtask architecture-check` prints the report and writes
`<target>/klibs-rules/violations.txt`, `<target>/klibs-rules/policy.txt` and `<target>/klibs-rules/api-ownership.txt`; it exits 0 when clean, 1 with findings, 2 on
a configuration, metadata or write error. No test writes a report file.

The ownership report is advisory: `CANDIDATE`, `RETAIN`, `UNKNOWN`, `UNCERTAIN` and `EXTERNAL` records provide evidence
for a human ownership decision and do not fail the architecture check. Existing callers may keep `invoke()` and
`write()`; analysis does not write files, and the original writer API still writes the two policy reports.

## Configuration

Every key, with the table it may sit in. Any other key, a key in the wrong table, or a value of the wrong shape stops
the run with exit code 2 and the file and line.

| Key | `[workspace.metadata.klibs-rules]` | `[package.metadata.klibs-rules]` | Value |
|---|---|---|---|
| `product-kind` | required | — | `"app"` or `"platform-plugin"` |
| `platform-words` | yes | — | the host frameworks' words; no built-in list |
| `prefix` | yes | — | the crate-name prefix; derived from the member names when absent |
| `entry-point-words` | yes | yes, adds to the workspace list | added to `command`, `event`, `route`, `job`, `service` |
| `technology-words` | yes | yes, adds | added to the words derived from dependencies |
| `plural-concepts` | yes | yes, adds | added to `news` and `settings` |
| `extension-kinds` | yes | yes, adds | kinds whose files hold `impl <Type>` blocks of types declared elsewhere |
| `role-kinds` | yes | yes, adds | the project's role kinds: kinds named by the last word of a class (`limiter`, `registry`); a word that is empty or not a valid Rust identifier, `test` or `fixture` (the block names of the crate root), a fixed kind, an entry-point word or a technology word is a configuration error |
| `lifecycle-traits` | yes | — | default `["Lifecycle"]` |
| `root-module` | yes | — | default `"RootModule"` |
| `role` | — | yes | one of the 15 roles below |
| `composition` | — | `impl-ui` only | `"local"` or `"host"` |
| `implementation-access` | — | `impl-ui` only | `"siblings"` or `"none"` |
| `host-items` | — | `entry-point` only | items a host loads by name |
| `checks.disabled.<RuleId>."<subject>" = "<reason>"` | workspace findings only | crate findings only | one subject of one rule |
| `checks.allowed-dependencies."<crate>" = "<reason>"` | — | yes | one dependency edge; it still counts for cycles |

A manifest exception naming an unknown or retired rule id, or a rule with no finding the table can hold, is a
configuration error. The Cargo configuration read is the one Cargo uses at the workspace root: `.cargo/config` when
it exists, `.cargo/config.toml` otherwise; parent directories and `$CARGO_HOME/config.toml` are out of scope.
`cargo metadata`, `rustfmt` and the toolchain checks use the cargo named by `$CARGO`, or `cargo` on `PATH` when
`$CARGO` is unset (an `xtask` binary started by hand or from an IDE), and the rustfmt beside it, or `rustfmt` on `PATH`.

## Roles

A crate's role is its declared `role`, else the first matching row. All fifteen rows participate in dependency and
placement checks; `core` and `impl-ui` require an explicit declaration.

| # | Role | Detected when not declared |
|---|---|---|
| 1 | `tooling` | the package is named `xtask` |
| 2 | `proc-macro` | its lib target is a `proc-macro` |
| 3 | `entry-point` | under `instances/`, or no Rust library target (`lib`, `rlib`, `dylib`, `proc-macro`): a cdylib-only or staticlib-only crate is an entry point, since no crate can use it (E0433) |
| 4 | `core-ui` | `modules/core/ui` or below it |
| 5 | `core-platform` | `modules/core/<word>` named in `platform-words` |
| 6 | `core-api` | another path below `modules/core` |
| 7 | `core` | declaration only |
| 8 | `contract` | final directory segment `contract` |
| 9 | `storage-contract` | final directory segment `dao` |
| 10 | `api` | final directory segment `api` |
| 11 | `impl` | final directory segment `impl` |
| 12 | `impl-ui` | declaration only |
| 13 | `platform` | final directory segment named in `platform-words` |
| 14 | `technology` | final directory segment matches a technology word of a non-development external dependency |
| 15 | `context` | remaining crates |

## Exceptions

A finding is fixed by the documents. An exception is written only for `Unanalysable`, a suspected false positive
reported to this repository (the reason names the report), or existing code a document keeps during an approved
rollout (the reason names that document line), and the user approves each one (`notes.rs.md`). There is no baseline.

- **Attribute form**: `#[cfg_attr(klibs_rules, expect(RuleId, reason = "…"))]`. `[workspace.lints.rust]` declares
  `unexpected_cfgs = { level = "warn", check-cfg = ["cfg(klibs_rules)"] }`; nobody passes `--cfg klibs_rules`.
- **Anchors**: a finding about a node is expected by an outer attribute on that node or on a node that contains it
  (an `impl`, a trait or a struct covers its members); the innermost expectation takes it. A finding about a module is
  expected by an outer attribute on its `mod` declaration or an inner attribute at the top of its block or file, never
  by one on a node inside it. An outer attribute on a `mod` declaration also takes the rule's findings anchored at that
  declaration node itself (a `mod x;` whose file is missing), never nodes inside the module. A finding about a crate,
  an edge or the workspace takes a manifest entry keyed by its exact subject.
- **Meta findings**, which take no exception: `MalformedExpectation` (a shape other than the one above, a missing,
  non-literal or blank reason, a position that cannot hold any finding of the rule, a second expectation of a rule on
  one node), `UnknownRule`, `RetiredRule` (the message names the new id), `UnfulfilledExpectation` (nothing to
  suppress; for a rule this version does not run, or one that reads platform crates while `platform-words` is empty,
  the message says so). `Unanalysable` takes the attribute exception at its anchor.
- **Not expectable**: `ForbiddenComment`, `UnitDependencyCycle`, `EntryPointDependency`, `LintConfig` and the meta
  findings.
- **Retired ids** (0.2.0; none retired since 0.1.1): `ActualFileSuffix` → `PlatformFileSuffix`, `CallbackFlowShape` → `CallbackBridgeShape`,
  `ClockSystemOutsideDi` → `SystemClockOutsideDi`, `CompanionFactoryFunction` → `AssociatedFactoryFunction`,
  `ConstantOutsideCompanion` → `ConstantOutsideImpl`, `DataClassContainsFunctions` → `ModelContainsFunctions`,
  `DataClassDefaultValue` → `StructDefaultValue`, `DataClassShouldBeImmutable` → `ModelShouldBeImmutable`,
  `GlobalCoroutineUsage` → `DetachedTask`, `ImplicitItParameter` → `ClosureParameterName`, `ImpureUpdateLambda` →
  `ImpureUpdateClosure`, `KDocFiller` → `RustdocFiller`, `KDocHtmlTag` → `RustdocHtmlTag`, `LambdaSingleIfGuard` →
  `ClosureSingleIfGuard`, `LocalOrInnerClass` → `LocalItem`, `NamespaceObject` → `NamespaceType`, `NestedDataClass` →
  `NestedDataStruct`, `RedundantVisibilityModifier` → `VisibilityTier`, `RolelessClassName` → `RolelessTypeName`,
  `RunBlockingInTest` → `BlockOnInTest`, `RunBlockingOutsideBoundary` → `BlockOnOutsideBoundary`, `SealedFamilyFile` →
  `EnumFamilyFile`, `StateFlowTimeout` → `WatchTimeout`, `TrivialLambdaReference` → `TrivialClosurePath`.

## Report

One line per finding, sorted by path, line, rule id and subject:

```text
<RuleId>\t<path>[:<line>]\t<subject>\t<message>
```

`path` is relative to the workspace root; `subject` is the key a manifest exception uses (a crate name, `.` for the
workspace, an item or module path through the import name). Accepted exceptions are printed the same way with the
reason in place of the message, at the exception's own location.

## Required configuration

`render_required_configuration()` renders the `[workspace.lints]` block and `clippy.toml` byte for byte; this
repository's root uses them; the clippy lints include `let_underscore_must_use`, `unused_result_ok` (`.ok();` on a
`Result`, which `ResultConsumption` proves only for its closed list of foreign calls), `wildcard_imports` and
`enum_glob_use`; the `clippy.toml` keys include `warn-on-all-wildcard-imports = true`. `LintConfig` reports a required lint that is missing or lowered (`allow`, `expect`, a
group set to `allow` with a priority not below the lint's own), a `check-cfg` without `cfg(klibs_rules)`, a missing or
false `clippy.toml` key, a missing threshold, a missing `disallowed-*` or `await-holding-invalid-types` path, a `std::sync` entry of
`disallowed-types` without a non-blank `reason`, `allow-print-in-tests` or `allow-dbg-in-tests`, a `clippy.toml` in a member directory or two at the root, a member
without `[lints] workspace = true`, a root without `resolver = "3"` (or the Cargo configuration's
`incompatible-rust-versions = "fallback"`), a toolchain file pinning `nightly`, `beta`, a stable below 1.93.0 or a dated `stable-YYYY-MM-DD` (4-2-2 digits) before 2026-01-22 (a host-triple suffix such as `1.93.0-x86_64-unknown-linux-gnu` is stripped first, and counts only when its first segment is a target architecture), a
nightly-only `rustfmt.toml` key for the running rustfmt, `--cfg klibs_rules`, `--cap-lints`, `-A` or `--allow` in
`rustflags`, an `[unstable]` table, `cargo-features` and `[env] RUSTC_BOOTSTRAP` in the Cargo configuration with any value but `-1`. Extra lints are allowed: the project's configuration wins.

Lints that fail a Good example or a kept layout, and so are advice against, not a check: `arbitrary_source_item_ordering`,
`absolute_paths`, `tests_outside_test_module`, `min_ident_chars`, `clone_on_ref_ptr`, `renamed_function_params`,
`module_name_repetitions`, `redundant_pub_crate`, `pub_use`, `missing_docs`, `missing_errors_doc`, `missing_panics_doc`,
and the groups `clippy::restriction`, `clippy::pedantic` and `clippy::nursery` as a whole.

The floor is 1.93.0; the library declares `rust-version = "1.93"`.

## Rules in 0.2.0

| Id | Anchor | Reports |
|---|---|---|
| `LintConfig` | none | every missing or lowered entry of the required configuration (above) |
| `RootSources` | workspace | a `[package]` table in the root manifest; a `src/` at the root (`arch-package.rs.md`, Units and Cargo) |
| `UnitReadme` | crate | a member without a `README.md` of its own (an inherited workspace `readme` does not stand in for it), a README whose sentence count is not one (counted as Kotlin counts), `readme` set to another path or `false` (`arch-package.md`, Units 8) |
| `NestedUnit` | crate | a member directory nested inside another member (`arch-package.md`, Units 7) |
| `ContainerSources` | workspace | a container holding a manifest, source directory or Rust file, or a child directory that holds files and has no member at or below it, or an immediate child directory with no member at or below it that the directory walk skips by name (`resources`, `build`, `node_modules`, a hidden directory) (`arch-package.md`, Units 7; `arch-package.rs.md`, Units and Cargo) |
| `UnitPath` | crate | tooling outside `xtask`, entry points outside `instances/<name>`, other members outside `modules/`, a `shared` path segment, an unregistered `xtask` (`arch-package.md`, Units 3; `arch-package.rs.md`, Units and Cargo) |
| `ProjectDirMatchesPath` | workspace | a wildcard in a workspace member path (`arch-package.rs.md`, Units and Cargo) |
| `RootBuildSubprojects` | crate | literal shared package settings or dependencies instead of workspace inheritance (`foo = "1"` and `foo = { version = "1" }` are one requirement; a renamed dependency, `alias = { package = "foo", … }`, is compared with the root by its package name); profiles, patches or replacements in a member (`arch-package.rs.md`, Units and Cargo) |
| `CrateTargets` | crate | custom library or binary roots; a library target and a binary target in one crate unless its role is `entry-point` (`src/lib.rs` plus one `src/main.rs`; a tooling crate such as `xtask` is `src/main.rs` alone); multiple binaries, `src/bin/` or a `test` target whose path is outside `tests/` (`arch-package.rs.md`, Units and Cargo) |
| `UnitDependencyCycle` | none | a strongly connected component of normal, build and development edges containing a development edge (`arch-context.md`, Direction and enforcement; `arch-package.rs.md`, Units and Cargo) |
| `EntryPointDependency` | none | any dependency on an entry point or a member without a Rust library target (`arch-package.md`, Units 1; `arch-package.rs.md`, Units and Cargo) |
| `RootPackagePath` | crate, workspace | a name other than `<prefix>-<path without its container, / → ->`; `[lib] name` set; a crate named `std`, `core`, `alloc`, `proc_macro` or `test` in either spelling (`arch-package.rs.md`, Units and Cargo) |
| `PublishedVersion` | crate | `version` (literal or inherited) with `publish = false` or `publish = []`, and `publish` true or a registry list without `version` (`arch-package.rs.md`, Units and Cargo) |
| `XtaskAlias` | workspace | an `xtask` member without `[alias] xtask = "run --package xtask --"` in the workspace Cargo configuration (the array form `["run", "--package", "xtask", "--"]` is the same alias) (`arch-package.rs.md`, Units and Cargo) |
| `ShellAutomation` | workspace | a `*.sh`, `*.bash`, `*.zsh`, `*.ksh`, `*.fish`, `*.csh`, `*.ps1`, `*.bat`, `*.cmd`, `Makefile`, `makefile`, `GNUmakefile`, `Makefile.toml`, `justfile`, `Justfile`, `.justfile`, `.Justfile`, `Taskfile.yml`, `Taskfile.yaml` or any other file (a dotted name included) whose shebang runs `sh`, `bash`, `zsh`, `dash`, `ash`, `ksh`, `fish`, `csh` or `tcsh`, at the workspace root or directly in a member directory; deeper in the tree, such a file that runs `cargo` (also after a shell keyword: `if`, `elif`, `while`, `until`, `then`, `do`, `else`, `!`, `{`, `}`). Approximate: what counts as build automation is the reviewer's call (`arch-package.rs.md`, Units and Cargo) |
| `DependencyFloor` | workspace | `Cargo.lock` resolving a crate below the supported floor recorded by this library (`arch-domain.rs.md`, Rust: domain building blocks) |
| `TimeCrate` | workspace | members that together declare more than one of `chrono`, `time` and `jiff`, including development dependencies and renamed packages |
| `PlatformSourceSetUnit` | crate | a member, other than an entry point, whose last path segment is a target word (documented resolution policy; `dragonfly`, `solaris`, `fuchsia` and `haiku` count only when the crate's own cfgs or target dependencies name them), as Kotlin reports a unit named after one of its eight targets (and not the root or an entry point) (`arch-package.md`, Units 5; `arch-package.rs.md`, Modules and directories) |
| `UnitDependency` | edge (manifest) | normal/build member edges forbidden by the dependent role, fake-feature boundary (a crate's own `fake = ["dep/fake"]` feature counts only when its `default`, or a `[dependencies]` edge of another crate that enables it, reaches the fake), tooling or proc-macro boundary (`arch-context.md`, Direction and enforcement; `arch-package.rs.md`, Tests) |
| `ApiUnitDependency` | edge (manifest) | dependencies forbidden by the architecture's api role row (`arch-context.md`, Direction and enforcement) |
| `TechnologyDependency` | edge (manifest) | a dependency on an external crate whose word is the word of a `technology` crate of the workspace, declared by a crate other than that `technology` crate and the entry points (`arch-context.md`, The dependency graph) |
| `PluginImplUnit` | crate | under `product-kind = "platform-plugin"`, a crate of role `impl` or `impl-ui`, and a sibling of an `api` crate whose role is neither `platform` nor `api`; each `context`-role unit of a directory below the container that holds two or more `context` units and no `api`, `contract` or `storage-contract` unit (units of other roles in that directory are not reported by this check and do not count toward the two) (`arch-package.md`, Units 4) |
| `ApiUnitPlatformDependency` | edge (manifest) | a platform crate (documented resolution policy) declared by a plugin's `api` anywhere, by an app's `api` outside `[target.'cfg(…)'.dependencies]`, or by `core-api`, `storage-contract` or `contract` anywhere (`arch-package.rs.md`, Units and Cargo) |
| `PlatformDependencyTable` | edge (manifest) | a normal dependency declared outside `[target.'cfg(…)'.dependencies]` whose every import, path reference, macro-call path and attribute path in the dependent crate sits under a `cfg` predicate that implies a target atom (`all` if any child implies one, `any` only if every child does, `not` never) (`arch-package.rs.md`, Modules and directories) |
| `DocHiddenUse` | node | a `#[doc(hidden)] pub` item (including a constant, static or union) or a `pub` inherent impl or trait member that no other workspace crate names or implements; a published library crate is checked as well (`arch-package.rs.md`, Visibility) |
| `DiModuleVisibility` | node | a `pub` DI module that no other workspace crate names (`arch-package.rs.md`, Visibility) |
| `RootFileShape` | node, file | a root item outside the ordered module tree and platform items, modules in file modules, an invalid fixture block, a `test` block that is not private or an undeclared Rust file; proc-macro exports are allowed, and in an `entry-point` root a `static` named in `host-items` or carrying `#[no_mangle]` or `#[export_name]` is a platform item; in a crate that has both `src/lib.rs` and `src/main.rs`, `src/main.rs` holds only private `use` items and `fn main` (any `mod`, inline or not, and any other item is reported: the tree is declared in `src/lib.rs`) and `src/lib.rs` is checked as any library root, while `src/main.rs` alone keeps the root-file rules and may hold the tree (`arch-package.rs.md`, Modules and directories) |
| `ModuleVisibility` | module | visible file modules or inline visibility inconsistent with reexports and test boundaries (`arch-package.rs.md`, Modules and directories) |
| `TestMirror` | node, module, crate | tests outside the mirrored test tree (a `cfg` that requires `test`, such as `cfg(all(test, unix))`, counts; `cfg(any(test, …))` does not), a test without a source owner or behaviour, a misplaced test block or path attribute; `tests/it/main.rs` is allowed for a published crate (`version`, not `publish = false` or `[]`) and for a proc-macro crate: its blocks mirror `src/`, its files get the same checks (`_test.rs`, mirrored source, behaviour) and the source reader reads `tests/it` for both; any other `tests/` content is reported (`arch-package.rs.md`, Tests) |
| `ForbiddenPackageName` | module | a module named in `arch-package.md`'s list, `prelude` (a file module too), `composable`, `theme`, `viewmodel`; `std`, `core`, `alloc`; a raw identifier; `test` or `fixture` other than the two blocks; `bin` directly under `src/` (`arch-package.md`, Concepts and kinds 10; `arch-package.rs.md`, Modules and directories) |
| `PlatformPackage` | module | platform names used as modules in a crate with target predicates or target dependencies; the fixed word list holds only words with no ordinary meaning (`x86_64`, `aarch64`, `wasm32`, `uefi`, …), and `arm`, `none`, `hexagon`, `haiku`, `dragonfly`, `solaris`, `fuchsia`, … count only when the crate's own cfgs name them (`arch-package.rs.md`, Modules and directories) |
| `KindContent` | node | in the source tree, an item a kind does not hold: a function, `static`, struct, enum, union or `const` in `api`; a function or `static` in `model`; a function, `static`, enum, union or `const` in `usecase`; in `event`, a free function or `static`, and a struct, enum or union named neither `…Listener` nor `…Event`, except a plain-data type (no hand-written `impl` holding a function; an `impl` under `cfg(test)` or in the test or fixture tree does not count) in a file that declares a type so named; items under a target `cfg` (one file per target) are exempt (`arch-package.md`, Concepts and kinds 3; `arch-package.rs.md`, Entry points, use cases and the platform) |
| `ImplementationPlacement` | node | a struct or enum in a technology module that implements a workspace `api` trait of another concept (`arch-package.md`, Contracts and implementations) |
| `ForeignErrorType` | node | a free function or inherent method in a `model`, `usecase` or `util` kind of the source tree whose `Result<T, E>` names a foreign `E` (`std::io::Error`, `Box<dyn std::error::Error>`); trait-impl methods keep the trait's signature |
| `ReExportShape` | node | missing/duplicate/renamed/glob kind re-exports, re-exported private items, a re-export of a kind block's item in any other block, an ancestor block included, public re-exports without external users, crate-private re-exports whose item has external users, and a `pub(super)`, `pub(self)` or `pub(in …)` re-export (`arch-package.rs.md`, Modules and directories) |
| `ImportThroughFileModule` | node | a `use`, type or expression path that walks through a file module, except the kind block's own re-exports (`arch-package.rs.md`, Modules and directories) |
| `ApiConfiguration` | node | `pub use` or `pub(crate) use` of a path whose first segment is an external crate (`arch-package.rs.md`, Units and Cargo) |
| `KindShape` | module | a concept block holding a file module (at that file module's declaration; in a technology, platform or core-platform crate a file module of the test tree or of `tests/it` passes when the `src` file it mirrors passes: `test/<x>_test.rs` and `tests/it/<x>_test.rs` mirror `src/<x>.rs`; a fixture file has no source twin); a block holding both files and blocks; a kind block inside a kind block; an empty kind block; a block that holds files and is no kind; a role-kind module (a word of `role-kinds`) is a kind: its structs, enums and unions end with the word in PascalCase (`limiter` → `VoteCooldownLimiter`), and it holds types (struct, enum, union, type alias), their impls and `use` items only (every other item is reported: a free function, trait, trait alias, `static`, `const`, `macro_rules!`, an item macro call, `extern crate`, an `extern` block); a module in the test, fixture or `tests/it` tree is not checked; a role-kind module in an app `api` crate is reported by the crate-role check (`arch-package.md`, Concepts and kinds 1 and 3) |
| `KindImportDirection` | node | imports forbidden by the architectural kind directions, checked per kind module whichever concepts the two belong to, including framework/registration boundaries, respecting cfg and test boundaries: `model` imports `model`; `api` imports `model`; `util` imports `model`, `util` and the data types of `api`, not a trait of `api`; `usecase`, `stub`, technology kinds and entry-point kinds also import role kinds; a role kind imports `model`, `util` and role kinds; `fake` imports `api`, `model`, `util`, a `fake` module of the test or fixture tree and the trait of its crate's `api` or `di` that its file implements; `di` imports everything and only `di` imports `di` (`arch-context.md`, Direction and enforcement) |
| `PackageDependencyCycle` | node | each strongly connected component of more than one kind module among production edges, under compatible cfg predicates, with one `file:line` per pair (`arch-context.md`, Direction and enforcement) |
| `PlatformImport` | node | in the `src` tree (the test and fixture trees are not checked), in `model`, `api` and `usecase` modules and in every module of a plugin `api`, `core-api`, `storage-contract` or `contract` crate, a path to a platform crate or a technology crate (a crate whose word names a technology module or `technology` crate of the workspace, or is in `technology-words`) (`arch-domain.md`, What the model may import; `arch-package.md`, Contracts and implementations) |
| `ContractPlacement` | node | a trait outside `api`, except a `<Type>Factory`, a `<Type>Ext` in `util`, a use-case trait in `usecase` (P3: app `api` crates only), a state holder's trait beside its `Default…` in an entry-point kind, and a registration trait in its mechanism's module; an `impl` of a crate trait inside `api`; a `stub` module holding anything but structs that implement crate traits. P3: an app use-case trait not in the owner's `api` `usecase`, or its `Default…` not in the `impl` `usecase`. Approx registration shape: every method synchronous, required, returning `()`, taking `&self` plus parameters whose types all come from external crates; such a trait may sit in an entry-point kind or in mixed `core`, but not in `model`, `usecase` or `util` of a core crate (`arch-package.md`, Contracts and implementations) |
| `UseCaseShape` | node | placement/naming/public-surface checks, use-case chaining, supported plain contract passthrough and contract field ownership; semantic business-rule necessity remains review (`arch-domain.md`, Use case; `arch-package.md`, Units 6) |
| `UseCaseAbstraction` | node | a trait named `*UseCase` outside the `usecase` kind of an app's `api` crate (`arch-package.md`, Units 6) |
| `UtilShape` | node | a struct, enum, union, type alias, const or static in `util` (a const or static passes when a function or an `impl` of its file uses its name, such as an `<Type>Ext` trait with its impl), and an inherent `impl T` outside T's file and outside `util`, except in `extension-kinds`; in the `src` tree only (the `util` blocks of the test and fixture trees and of `tests/it` are not checked), a function in `util` (free, or a method of an out-of-file inherent impl) with a parameter that names a project trait declared in an `api` kind: `&dyn Trait`, `Arc<dyn Trait>`, `Box<dyn Trait>`, `impl Trait`, or a generic bounded by it (inline or in `where`); `<Type>Ext` trait impls, a type alias that hides `dyn Trait` and a module-path use of an `api` item are not examined (`arch-package.md`, Concepts and kinds 3; `arch-package.rs.md`, What goes in one file) |
| `UtilFileName` | node, module | generic utils/helpers/extensions filenames and supported util receiver/predicate naming mismatches, where the owner of a `<Type>Ext` may be a primitive, slice, array or reference type; generated files skipped (`arch-package.rs.md`, What goes in one file) |
| `UtilFunctionAsExtension` | node | the form of a `util` rule follows its signature, in the `src` tree only (the `util` blocks of the test and fixture trees and of `tests/it` are not checked). The cfg variants of one model type (the same name in the same module under exclusive `cfg`s) count as one type. A free function whose model-typed parameters (`&`, `&mut`, `Box`, `Rc` and `Arc` removed; `Vec`, `Option` and `Result` are not; a type named `…Id` is no model type) are all of one type is a method of that type; with two or more different model types it is reported only when it returns the type of one of them (`T`, `Result<T, _>`, `Option<T>`: a transition is a method of the type it returns), and is otherwise a free function named after the decision. A method of an out-of-file inherent `impl` of a model type in `util` is reported when it has a parameter of another model type and does not return its own type (`Self`, the type, `Result` or `Option` of it); a method whose model-typed parameters are all of its own type passes. `create_*` and `parse_*` returning `Result` of a model are exempt; methods of trait impls are not examined (`arch-domain.md`, Rules live in `util`) |
| `PluralPackageName` | module | a block name ending in `s` but not in `ss`, `us` or `is`, unless in `plural-concepts` or one of Kotlin's two built-in words, `news` and `settings` (README); the singular words `alias`, `atlas`, `bias`, `canvas`, `christmas`, `gas` and `pancreas` are not plural (a closed list: another singular word in `-as` goes into `plural-concepts`) (`arch-package.md`, Concepts and kinds 9) |
| `DiPlacement` | node, module | misplaced wiring structs, missing DI for proven dependency assembly/resource ownership, and DI in proven contracts/models-only crates; a DI module in the `src` tree of any target of the crate (`src/lib.rs` of a lib-plus-main entry point; a DI-shaped type in the test, fixture or `tests/it` tree does not count) satisfies the crate's own-DI-module requirement for the constructions in `src/main.rs` (`arch-package.md`, Concepts and kinds 3; `arch-package.rs.md`, DI modules) |
| `RootModulePlacement` | node | a `root-module` struct outside `src/di/root_module.rs` of an `entry-point` crate, and more than one in a crate (`arch-package.md`, Units 1; `arch-package.rs.md`, Entry points, use cases and the platform) |
| `EntryPointShape` | node, module | in an `entry-point` root, blocks other than `di`, `model`, `error` (the framework failure type's kinds) and the fixture and test blocks; `fn main` with any visibility modifier; a non-`pub` host item, or function or `static` with `#[no_mangle]`, `#[unsafe(no_mangle)]` or `#[export_name]`, in a cdylib or staticlib (`arch-package.md`, Units 1; `arch-package.rs.md`, Entry points, use cases and the platform) |
| `EntryPointVisibility` | node | a `pub` type in an entry-point kind, except an app `api` state holder's interface and a `pub` event type in the `event` kind of a `platform` crate that carries a host derive or trait: a derive or trait of a foreign, non-workspace crate, or, when `platform-words` is set, only one that resolves to a platform identity (the same test as `EventPlacement`); a type that a binary target of the same crate uses through the crate's name (`shop_cli::command::X` from `src/main.rs`) may be `pub` (`arch-package.rs.md`, Visibility) |
| `ImplementationVisibility` | node | a `pub` struct or enum in a technology module that implements a crate trait, unless the module is `stub` or `memory` (`arch-package.rs.md`, Visibility) |
| `CoreShape` | node, module | a `usecase` kind, a `*Repository` trait, a `*Repository` struct or enum or an `impl` of a `*Repository` trait (the last two in `src` only), in a `core`, `core-api`, `core-platform` or `core-ui` crate (`arch-package.md`, Units 1) |
| `ContractUnitShape` | node | in a `contract` crate, a struct or enum lacking `Serialize` or `Deserialize`, a type alias, a union, a `static`, and any function body; consts are route constants and allowed (`arch-package.rs.md`, Units and Cargo) |
| `ErrorPlacement` | node | an enum or struct deriving `Error`, holding a hand-written `impl` of `std::error::Error` or `core::error::Error`, or named `*Error`, or an enum named `*Result`, or a crate enum that a function or method of the crate returns as the `E` of `Result<T, E>`, outside `model`, one in an old `error`, `errors` or `exception` module included; in an entry-point crate, a workspace error enum outside the crate's `model` returned by a `route` function or inherent method (`arch-package.md`, Files) |
| `EnumFamilyFile` | node | a struct used only as a payload of one enum's variants but declared outside that enum's file (`arch-package.rs.md`, What goes in one file) |
| `NestedDataStruct` | node | a data struct used as a field type by exactly one type but not in its file, and a type returned to another type but not in its own file in `model`; a type declared in the file of its single field owner that a free function of that file, another type's `impl` or a use outside the owner also names; name counts from the crate index (`arch-package.md`, Files; `arch-package.rs.md`, What goes in one file) |
| `NestedEnum` | node | an enum used only through one type but declared outside its file; an enum declared in the file of its single field owner that a free function of that file, another type's `impl` or a use outside the owner also names (`arch-package.rs.md`, What goes in one file) |
| `FakePlacement` | node, crate | separate fake crates, incorrect fake tree/concept/visibility, missing fixture feature pairing, `fake` in the `default` feature list or reached from it through other features of the crate, a fake of another workspace crate's trait in the test tree, or duplicate test/fixture fakes; a path in `src/`, outside the test block and the fixture block, that resolves into the `fixture` module of another workspace crate, reported at each reference (the import and every use of it); the same path from `test/`, the crate's own `fixture/` or `tests/it` is not reported, and neither is a module-only import (`use other::fixture::x::fake;`), which names no item (`arch-package.rs.md`, Tests) |
| `InMemoryPlacement` | node | an `InMemory*` struct or enum outside a `memory` module, and an implementation of trait `T` in `memory` not named `InMemory<T>` (`arch-package.rs.md`, Tests) |
| `ImplPlacement` | node | an `impl` of a crate type outside that type's file, except: an inherent impl in `util/<t>_ext.rs`; `From` or `TryFrom` between a model type and a type declared in the file holding the impl (the technology representation's file); `From` or `TryFrom` between a model type and an external crate's type, in a technology module of that crate; P3: a framework's trait for an error enum, in the `error` kind of an `entry-point` crate (`arch-package.rs.md`, What goes in one file) |
| `UnsafePlacement` | node | an `unsafe` block, `unsafe fn`, `unsafe impl`, `unsafe trait`, `unsafe extern` block, or an `#[unsafe(…)]` attribute (`no_mangle`, `export_name`, `link_section`) outside a technology module, except the attributes in the root of an entry-point crate (`arch-package.rs.md`, Visibility) |
| `ListenerName` | node | a struct, enum or union implementing an external `*Listener` trait that is not named `*Listener`, or implements that trait outside `event`; unrelated external traits such as `Drop` do not establish listener intent (`arch-domain.md`, Domain event) |
| `EventPlacement` | node | a type named `*Event` outside `model`; in the `event` kind of a `platform` crate a type that implements a trait of another crate, by an `impl` or by a `#[derive(…)]` that resolves to a crate other than `std`, is the platform's event and passes; when `platform-words` is set, only an `impl` or a derive that resolves to a platform identity (a `platform` or `core-platform` workspace crate, or a crate named by a platform word) exempts it, so a `serde` derive or another crate's trait no longer does, and a host trait from a crate outside the platform words needs that word added (`arch-domain.md`, Domain event; `arch-package.rs.md`, Entry points, use cases and the platform) |
| `IntraDocLink` | node | an inline code span in a doc comment holding exactly an identifier (or `Type::member`), with or without a trailing `()`, that resolves in the item's scope and is not written as an intra-doc link |
| `VisibilityTier` | node | public reachability and indexed type/item users; `pub(super)`, `pub(self)` and `pub(in path)` on items, fields and methods; in an unpublished crate, a `pub` field or method of an externally used type whose name no other workspace crate uses (a tuple field counts as used when another crate writes its index after a dot, or the type's name, or the name of a `type` alias or `use … as` rename of the type, before a parenthesis, and inside a crate a `pub(crate)` tuple field counts as used when another file of the crate writes the name of a `type` alias or `use … as` rename of its type before a parenthesis); the users it indexes include the handlers of a `select!` macro; inferred widest visibility for fields/methods remains review; a `pub` method or field counts as used when another target of the same crate (the binary) names it (`arch-package.rs.md`, Visibility) |
| `AssociatedFactoryFunction` | node | an associated function without `self` on a model type other than `new`, wherever its `impl` is; a free `create_*` returning a proven model, directly or through supported Result/Option aliases, outside `util` or without `Result`; naming alone does not classify a helper as an aggregate factory (`arch-domain.md`, Factory) |
| `ModelContainsFunctions` | node | for a model type: more than one inherent `impl` in its file; in that `impl`, anything but associated consts and, when `new` calls `assert!` or the type's file holds `impl TryFrom<…> for` it (`std` or `core` `TryFrom`), `new` and accessors (one field read of `self`, also through argument-less `as_*`, `clone`, `copied` or `cloned` calls); any other method in the type's file; trait impls are not examined (`arch-domain.md`, Rules live in `util`; `arch-domain.rs.md`, Rust: domain building blocks) |
| `ModelShouldBeImmutable` | node | mutable self receivers (`self: Pin<&mut Self>` included) and field writes through supported typed model parameters, locals and reference aliases; arbitrary receiver inference remains review (`arch-domain.rs.md`, Rust: domain building blocks) |
| `ForbiddenImport` | node | a path to `chrono`, `time` or `jiff` outside technology modules, `technology` crates and `contract` crates |
| `ForbiddenComment` | none | region markers (`region`/`endregion` followed by the end, whitespace or `:`, with an optional `#` or `pragma`; `<editor-fold`, `</editor-fold>`, `MARK:`), separator lines and section headers, framed with `=`, `-`, `*`, `#`, `~`, `_`, `+`, `/` or a repeated non-ASCII symbol, or one-sided (`=`, `-`, `*` in doc comments; also `#`, `~`, `_`, `+` and a repeated non-ASCII symbol in other comments), in comments and doc comments (fences, thematic breaks in doc comments and Markdown emphasis runs aside); generated files skipped |
| `ClosureParameterName` | node | a one-letter closure parameter, including destructured bindings |
| `ExplicitItName` | node | an `it` binding, parameter, field, loop label, owned item or import binding, also inside the branches of a `select!` or `select_biased!` macro (tokio's and futures' grammar, and crossbeam's `recv(r) -> x`, `send(s, v) -> x` and `default(timeout)` branches, read for a bare `select!` or `select_biased!` and for the `tokio`, `futures`, `crossbeam` and `crossbeam_channel` paths only); trait-required names are preserved |
| `FunctionVerb` | node | a function starting with `get_`, `map`, `make`, `do`, `handle` or `process`; receiverless collection lookup names also report; trait methods and tests keep their names, as do `get`, `get_mut`, `get_unchecked` and `get_unchecked_mut` taking a receiver and a key parameter, and a method whose body is only its own field read (`self.<name>`, `&self.<name>`, `&mut self.<name>` or `self.<name>.clone()`, `<name>` being the method name) |
| `ImplementationName` | node | Impl/Interface suffixes, an `Impl` prefix (`ImplRatingRepository`; not `Implicit…`), interface I prefixes, and generic Database/Network/Local/Remote prefixes (ending at a word boundary) on proven implementations, those of a use-case trait in the `usecase` kind included, and on type aliases; correspondence with a full trait name remains review (`arch-package.md`, Contracts and implementations) |
| `RolelessTypeName` | node | as Kotlin does: a struct, enum, union or trait named `*Validator`; one named `*Factory` that is not the runtime-factory form; one named `*WarmUp` outside a `warmup` module and `*Migration` outside a `migration` module (`arch-package.md`, Concepts and kinds 3; `arch-domain.md`, Factory) |
| `KindNameInPackage` | node | utility suffixes, Model/Entity repetition in model kinds, API suffixes on contracts, and the word Service in any type name outside a `service` module (module path only, not the crate name); a type alias is checked like a declared type (`arch-domain.md`, Entity, value object, identifier; `arch-domain.md`, Domain service) |
| `BareIdentifier` | node | bare standard strings, UUIDs or integer primitives (including supported aliases and the wrappers `Option`, `Box`, `Rc`, `Arc`, `Cow<str>` and `NonZero*`) in ID fields, in an `ids` or `*_ids` field holding a `Vec`, `VecDeque`, `HashSet` or `BTreeSet` of a bare type, malformed typed-ID structs, a `type <Name>Id = <primitive>` alias in `model`, and single-property named model value objects (a one-field struct whose field is `id: <…>Id` is an entity and passes, and so does one that derives `Serialize` or `Deserialize`, also through `cfg_attr`, unless it carries `#[serde(transparent)]`, `try_from` or `from`: its field is the key of a file format, which a tuple struct would write without it); serialization alone does not exempt an id field of a domain model, while technology/wire contract boundaries do (`arch-domain.md`, Entity, value object, identifier) |
| `TestFunctionName` | node | a test name outside `given_<state>_when_<action>_then_<outcome>`; a test attribute counts when it resolves to `tokio::test` through a `use` |
| `CallbackDeclaration` | node | outside the test and fixture trees, as Kotlin does outside test sources: a function parameter or a struct field named `on_*` whose type, bare or inside `Option` or another single-value wrapper (`Mutex`, `RwLock`, `OnceLock`, `Cell`, `RefCell`, `OnceCell`), is a closure type (`impl Fn…`, `dyn Fn…` behind any pointer, an `fn(…)` pointer: Kotlin's function type) or a type parameter bounded by `Fn`, `FnMut` or `FnOnce`; a trait the project declares named `*Listener` or `*Callback` |
| `DeclarationOrder` | node | constants, type families, constructors or trait associated items out of their required order; a constant with a value inside a trait body (it belongs at the top of the file) |
| `CalleeBeforeCaller` | node | free-function callers above callees in files without type declarations, and own-type method ordering; includes self-qualified paths and proven local function aliases with lexical scope; constructors and mutual recursion are exempt; arbitrary dynamic call targets remain review |
| `OneTypeFamilyPerFile` | node, module | a mismatched file name, multiple behavioural types, unrelated secondary types (a type's reference to itself is no use) or free functions beside a type, a secondary struct or enum named `*Error` or deriving `Error` included; `<Type>Factory` is exempt (`arch-package.rs.md`, What goes in one file; `arch-package.md`, Files) |
| `ConstantOutsideImpl` | node | a type's constants outside or before the end of its inherent impl; a file module holding only constants beside the file of its owner type (the module name is the owner's snake-case name followed by `_` and a word, or the owner's file mentions the module) |
| `NamespaceType` | node | a struct without fields, or an enum without variants, with an inherent impl holding a function and no trait impl (`arch-package.rs.md`, What goes in one file) |
| `LocalItem` | node | `struct`, `enum`, `union`, `trait`, `impl` or `mod` inside a function body, closure or `async` block (`arch-package.rs.md`, What goes in one file) |
| `StructDefaultValue` | node | project defaults and omitted own-type fields through struct update syntax, also inside the handlers of a `select!` macro, with documented deserialization/constructor exceptions |
| `AnonymousTuple` | node | stored multi-element tuple types/structs/variants and proven tuple-returning APIs, including supported enumerate and `char_indices` elements, `iter`, `iter_mut` and `into_iter` over a typed std `HashMap` or `BTreeMap`, tuples built by `if`, `match` and block initialisers, scoped local iterator aliases and receiver-specific standard partition/unzip results with workspace extension-override guards; immediate local destructuring, including its type annotation on a `let` or on a closure parameter, passes; the handlers of a `select!` macro are read; unknown receiver return types remain review |
| `WildcardImport` | node | every `use` tree with a glob |
| `PlatformFileSuffix` | node | a `mod x;` under a single-key target predicate (documented resolution policy) whose name does not end in `_<value>`; a compound target predicate (`all`, `any`, `not`) on a `mod`; platform files whose re-exports differ in name or are not under the same predicate; a target predicate on anything other than such a `mod` and its re-export (`arch-package.rs.md`, Modules and directories) |
| `RustdocHtmlTag` | node | an HTML tag `<tag …>` or `</tag>` in a doc comment, hyphenated custom elements (`<my-widget>`) included, outside code spans and fenced blocks |
| `RustdocFiller` | node | fixed filler phrases (`this function`, `this method`, `this struct`, `this enum`, `this trait`, `this type`, `this module` or `this crate` followed by `is responsible for`, `allows you to` or `is intended for`; `returns the result of the operation`) and lexical restatement of parameter names; semantic paraphrases remain review |
| `BooleanOperatorWrapping` | node | an `if`, `while` or match-guard condition spanning several lines with a top-level `&&` or `\|\|` operand that is not a path, `!path` or literal |
| `TypeAliasOnOneLine` | node | a type alias, an `impl` associated type or a trait associated type with a default whose text from `type` to `;` spans several lines, also inside the items of an item-bodied macro (`some_macro! { type … }`); an alias with a `where` clause is skipped unless its type or its header (generic parameters and bounds) spans several lines (rustfmt always breaks the `=` and the `where` itself); anchored at the `type` line; generated files are skipped |
| `InitializerOnDeclarationLine` | node | a `let` with an initializer, a `const`, a `static`, an `impl` associated `const` or a trait associated `const` with a default whose initializer starts on a line after its `=`, also inside macro arguments, `select!` branches, the `[x; n]` form, item-bodied macros (`thread_local! { static … }`, its last `static` without `;` included) and statement-bodied macros (`stream! { let … }`, a final `let` without `;` included); a macro body is read through one interpretation only (expressions, the repeat form, items, statements, the first that parses; a body whose first expression starts with a `let`, as in `let ok = a || b`, is a statement body); anchored at the keyword line; assignments are not declarations; generated files are skipped |
| `ClosureSingleIfGuard` | node | a closure or `async` block whose whole body is one `if` without `else` around two or more statements |
| `OperatorBeforeTrailingCall` | node | a method call on a parenthesised arithmetic expression (one operator: write the `std::ops` method; two or more: name a `let`), and two chained calls of `add`, `sub`, `mul`, `div` or `rem` when the `std::ops` trait is imported (a call whose value is discarded and a chain rooted at `Type::f(..)` of a workspace type with an inherent method of that name are no operator) |
| `TrivialClosurePath` | node | chains of at least three path-only adapters, and proven trim/trim_start/trim_end adaptation of the items of a `&str` iterator (the receiver is a `&str` parameter or a local bound to a `&str`: a literal, a `trim` of a `&str` or an annotated `&str`; `lines`, `split_whitespace`, `split_ascii_whitespace`, or `split`, `rsplit`, `split_terminator`, `rsplit_terminator` with a character or string literal) followed by a compatible nongeneric safe workspace function accepting str, which can be two function-path steps; over any receiver, `clone`, `to_string` or `to_owned` of the item followed by one such workspace function; unsafe and target-feature targets are excluded; unknown coercions and effects remain review |
| `ImportOrdering` | node | use declarations after other items, separated into multiple blocks by a blank line or a comment (in inline module blocks, function and block bodies, and the root file too), or followed by a comment; a `use` at the start of a body conforms |
| `BlockOnOutsideBoundary` | node | `#[tokio::main]`; `block_on` (a `LocalSet`'s and a dependency crate's free function included), `Runtime::new` and `Builder::new_*` outside `fn main` of an `entry-point` root, except `block_on` in a technology module |
| `BlockOnInTest` | node | `block_on` (also a dependency crate's free `block_on`, such as `futures::executor::block_on`, and a `LocalSet`'s), `block_on_stream`, `LocalPool::new` and a `LocalPool`'s `run_until`, `Runtime::new` and `Builder::new_*` in the test and fixture trees |
| `IgnoredTest` | node | `#[ignore]` |
| `ExpectScope` | node | a lint expectation outside the prescribed lint list or smallest item, including inner and module-level expectations, and an expectation of an expression lint (`unwrap_used`, `expect_used`, `let_underscore_must_use`, `disallowed_methods`, `print_stdout`, `print_stderr`) on an `impl` or a `trait` |
| `DetachedTask` | node | proven spawned handles (`spawn`, `spawn_blocking` and `spawn_local`, also through `Handle::current()` or a runtime's `handle()`, and a `tokio::task::Builder` `spawn`, `spawn_blocking` or `spawn_local` whose handle is dropped, or passed to `drop(..)` or `std::mem::forget(..)`, after `?`, `.unwrap()` or `.expect(..)`) discarded directly (only `_ = …` among assignments; an assignment to a place stores the handle), explicitly dropped, or abandoned after supported local binding/branch analysis; in a test function a handle only aborted or polled with `is_finished` is abandoned; the handlers of a `select!` macro are read; arbitrary ownership and shutdown flows remain review |
| `WatchTimeout` | node | supported immediately-ready timeout futures (`timeout` and `timeout_at`, an `async` block of one awaited expression judged by that expression; a workspace `async fn` or `async` block suspending in `join!`, `try_join!`, `select!` or `select_biased!` awaits something) and tautological watch predicates; general readiness and predicate semantics remain review |
| `ImpureUpdateClosure` | node | captured state assignments, `&mut` borrows of a capture and supported captured collection mutations (an Option's `take`, `replace`, `insert`, `get_or_insert`) in atomic updates; known locking/I/O is checked where receiver identity is established (a blocking method on a receiver typed `std::fs::File`, `std::net::TcpStream` or `std::process::Command`, and the free functions `std::io::stdin`, `stdout`, `stderr`, `copy` and `read_to_string`, in `send_modify`, `send_if_modified` and `fetch_update` closures); also `std::thread::sleep` and the `tracing` and `log` event macros inside a `send_modify` or `send_if_modified` closure |
| `CallbackBridgeShape` | node | spawning (`tokio::spawn`, `spawn_blocking`, `spawn_local`, also as a method of a `Handle`, a `Runtime` or a `LocalSet`) or proven bounded-channel blocking_send inside a callback with source-proven registration/storage; synchronous combinators and unknown callable contracts do not establish callback intent; external registration semantics remain review |
| `BridgeCleanup` | node | `tokio::spawn`, `std::thread::sleep`, or `block_on` on a `Handle` or a `Runtime`, inside `fn drop` of an `impl Drop` |
| `ThreadPrimitive` | node | an `#[expect(clippy::disallowed_methods)]` covering a thread start outside a technology module; in a technology module, a discarded `std::thread::spawn` or `Builder::spawn` (statement, `let _`, `_ =`, `drop`); a handle that is returned, bound or the value of a block or `match` is not a discard; a handle assigned to a place is consumed, a `join()` in a loop body counts, and the `for` variable over a handle vector carries its handles |
| `DurationOverloadCall` | node | `Duration::from_*(<x>.as_millis() \| as_secs() \| as_micros() \| as_nanos() \| as_secs_f64() …)` round trips |
| `ResultConsumption` | node | discarded-result expectations outside a let-underscore statement or without a proven permitted sender method/type pair, a proven `Result` call (a workspace free or associated function, a method of a workspace type, or an awaited `async fn` of these; a `std::fs` module function; an awaited `tokio::fs` module function; an awaited `JoinHandle` or `tokio::spawn` family call) discarded by `_ = …;`, by a `let _name = …;` never read before the name is bound again (a typed `let` or a destructuring pattern binds it again), or by `.ok()` as a statement or under `_ =` or `let _ =`, and multiple direct question-mark steps in one let initializer, one expression statement, one `return` or the body tail of a closure; separate lets and deferred bodies are not combined; unknown receiver identity does not establish a discard exception |
| `DiModuleLifecycle` | node | unsupported lifecycle names/async teardown, supported child forwarding/order checks, borrowed-child ownership and task-scope cancellation; arbitrary external lifecycle semantics remain review |
| `RootModuleConstruction` | node | non-module construction/factories and supported service-starting work in root assembly or lifecycle methods, the arguments of a macro such as `vec![…]` included |
| `SystemClockOutsideDi` | node | a `SystemClock` constructed outside `di` and the test tree; a `SystemClock` declared outside a `system` module; a trait `Clock` that does not declare exactly `fn now(&self) -> SystemTime`; a production wall-clock read outside `impl SystemClock` (`SystemTime::now`, `UNIX_EPOCH.elapsed()`, chrono's `Utc::now`/`Local::now`, jiff's `Timestamp::now`/`Zoned::now`, time's `OffsetDateTime::now_utc`/`now_local` and `UtcDateTime::now`) |
| `TimePrimitive` | node | Instant in project fields/returns or supported local storage/send/log operations (the first element of a channel constructor's pair is a sender; `format!`, `format_args!`, `write!` and `writeln!` count like log macros), and numeric/text constants, named fields (also inside `Option`, `Box`, `Vec`, `VecDeque`, an array or a slice), parameters of free functions and inherent methods, and locals initialised by an integer literal, with explicit time-unit suffixes (`timeout_seconds: u32`; not in a technology module or a `contract` crate, not in a `serde`-derived type of an entry-point kind (`route`, `command`, …), and not a parameter of a trait-impl method or of a function returning `Duration`, where a boundary decodes the value); local elapsed-duration measurements pass, arbitrary temporal intent and external boundaries remain review |
| `ModelDerives` | node | a model struct without `Debug`, `Clone` and `PartialEq`, a `*Id` without `Debug, Clone, PartialEq, Eq, Hash`, and missing `Copy` when its inner type is proven Copy; a struct whose derive resolves to, or is written as, a derive named `Error`, or that implements `Error`, is skipped (`arch-domain.rs.md`, Rust: domain building blocks) |
| `ValueObjectShape` | node | validated-model field privacy/accessors (a field read, also through argument-less `as_*`, `clone`, `copied` or `cloned` calls) and constructor purity, plus supported parse-function location, signature (the error argument of its `Result`, after alias expansion, is a workspace enum) and single-construction flow; a model struct checked by a `TryFrom` impl in its file is a validated value object too: a `pub` or `pub(crate)` field is reported (a `<Type>Parts` struct is plain data), and so is a `parse_<type>` function outside `util` (`arch-domain.md`, Entity, value object, identifier; `arch-domain.rs.md`, Rust: domain building blocks) |
| `ValidatedModelDerives` | node | a model struct with a private field and an `impl TryFrom<…> for` it in its file that derives `Deserialize` (also through `cfg_attr`, nested to any depth) without `#[serde(try_from = "…")]`, or that derives `Default`: a derived deserializer or `Default` builds the value without its check (`arch-domain.rs.md`, Rust: domain building blocks) |
| `ContractShape` | node | contract supertraits (`Send`, `Sync` and `'static` pass, any other is reported; generics and where clauses are not reported), missing async_trait, non-BoxStream stream contracts, a returned future type (`BoxFuture`, `LocalBoxFuture`, `impl Future`, `Pin<Box<dyn Future>>`), a returned `tokio_stream::wrappers` stream, a returned `std::sync::mpsc` or `futures` `mpsc` receiver, a `BoxStream` whose first lifetime is not `'static` under any method name, a tokio `mpsc::Sender`, `mpsc::UnboundedSender`, `broadcast::Sender` or `watch::Sender` parameter or a `std::sync::mpsc` `Sender` or `SyncSender` parameter (`oneshot` passes), callback/task handles and external error types (a one-argument `Result<T>` resolved to an external alias included) (`arch-domain.rs.md`, Rust: domain building blocks) |
| `ErrorFamilyShape` | node | non-enum error families, missing Debug/thiserror::Error/error messages, non-boxed foreign source fields (a `#[source]` of a workspace family passes), proven foreign from/source payloads (an external type named `*Error` or with an `error` module in its path, such as `tokio::time::error::Elapsed`), and fields holding `Rc`, `Weak`, `Cell` or `RefCell` at any depth |
| `ErrorLogging` | node | supported caught errors logged without an error-chain field, logged repeatedly on the same supported path, or logged before propagation; arbitrary logger adapters and interprocedural exactly-once handling remain review |
| `CatchUnwind` | node | `catch_unwind` whose closure calls a crate-local item, or whose argument (also inside `AssertUnwindSafe`) is a path to a crate-local function or to a local closure that calls one, through a chain of local closures each followed once; a local closure is not taken for a crate-local function of the same name |
| `DocCommentScope` | node | mechanically redundant application/trait-implementation docs, malformed contract headings, missing published error/safety contracts and proven panic docs (`unwrap`, `expect`, `unwrap_err` and `expect_err` on a parameter; `panic!`, the assertion macros, `unreachable!`, `todo!` and `unimplemented!`), including supported caller-controlled indexing and default trait bodies; external reasons, additional contract information and unknown panic behavior remain review |
| `GlobalState` | node | a `static` typed `OnceLock`, `LazyLock`, `Mutex` or `RwLock`, also inside std `thread_local!`; `lazy_static!`; installing a logging subscriber (`set_global_default`, `.init()` on a subscriber builder) outside `fn main` |
| `DiModuleShape` | node | DI declarations outside the permitted struct/trait forms, service construction outside new, generic contract parameters (on the impl, or on an inherent `fn` through its generics, where clause or `impl Trait` parameter, bounded by a workspace trait; on a use case every inherent `fn` but `new`, which `UseCaseShape` reads), unsupported sharing/deferred storage (a service is shared when it sits in an `Arc` or `OnceLock`, also as the element of a `Vec` or `VecDeque` or the value of a `HashMap` or `BTreeMap`) and contract injection despite an accessible compatible provider module; a DI trait's contract trigger and external semantics remain review |
| `ConstructorDependency` | node | supported construction/factory calls of proven replaceable dependencies outside composition boundaries, a free function included unless it sits in `di` or in the module that declares the constructed type; data containers and host root assembly are preserved (`arch-domain.md`, Factory) |
| `MainShape` | node | entry main runtime/root enable/disable structure and supported awaited shutdown wait, one level inside an awaited workspace `async fn`, free or associated, included; `<CancellationToken local>.cancelled().await` and an awaited `async` method of a workspace type whose body awaits the shutdown signal (one level) also count; arbitrary runtime control flow remains review |
| `BlockingCallInAsync` | node | `std::thread::sleep`, `std::fs::*` (every method of a `std::fs::File` included) or `std::sync::mpsc::*` inside an `async fn` or `async` block |
| `BroadcastLag` | node | terminating while-let, supported match loops and `if let Ok(…)` loops whose `else` breaks or returns, on proven broadcast receivers, and `<broadcast::Receiver>.recv().await?` inside `loop`, `while` or `for`; Closed termination and continuing Lagged handling pass |
| `SelectCancelSafety` | node | a branch future of `tokio::select!` inside a loop that calls `read_exact`, `read_to_end`, `read_to_string`, `write_all`, `lock`, a tokio `RwLock`'s `read` or `write`, `Semaphore::acquire` or `Notify::notified`, also through a workspace callee followed one level and analysed in its own crate and module |
| `JoinSetBehindLock` | node | a type `Mutex<JoinSet<…>>` or `RwLock<JoinSet<…>>` |
| `BlockingLockInSync` | node | `blocking_lock()`, `blocking_read()` or `blocking_write()` in `fn drop` or in a synchronous `on_enable` or `on_reload` method of an `impl` (not `on_disable`, which runs after `block_on`, and not a free function); in `fn drop` also `recv` or `recv_timeout` on a `std::sync::mpsc::Receiver`, `blocking_recv` on a tokio `mpsc` or `broadcast` receiver and `blocking_send` on a tokio `mpsc::Sender` |
| `CriticalSectionWrite` | node | multiple writes or trailing operations in supported guard scopes, including supported collection mutations (deque ends, the sort family, `mem::take`, `mem::replace` and `mem::swap` of a guard place) and nested branches; arbitrary panic atomicity remains review |
| `LockChoice` | node | proven integer/Boolean locks and supported RwLock patterns (a tokio `RwLock` whose guard no `.await` crosses included); general access-contention semantics remain review |
| `TestTimePaused` | node | known virtual-time operations without a paused test, also one level inside a called workspace `async fn` (free or associated), incompatible runtime flavor, and missing tokio test-util; deeper helper timing remains review |
| `ModelRuleTest` | node | `#[tokio::test]` with no await that directly exercises a proven pure model rule (a local bound to a non-model function path is not a model call); a `util` path alone does not classify an integration test |
| `Unanalysable` | node, crate | a root file or file module that does not parse, a missing module file, `include!`, `cfg_attr(…, path = …)`, an item macro call in the root file, except a macro called by a path through a non-workspace dependency of the crate whose arguments are plain identifiers (`pumpkin_plugin_api::register_plugin!(Plugin);`) in a crate with no Rust library target: no crate can name what such a crate generates |
| `UnfulfilledExpectation` | none | an expectation with no finding in its permitted scope |
| `UnknownRule` | none | an exception naming no catalogue rule |
| `RetiredRule` | none | an exception naming a retired id; the finding names its replacement |
| `MalformedExpectation` | none | malformed, duplicated, misplaced or reasonless expectations |

All catalogue IDs above are active. Descriptions summarize retained checks; the documents define their exceptions.
Type and call checks use resolved Cargo identities and compatible cfg branches. Unknown receiver inference does not
replace rustc type checking. Review portions are retained as review rather than empty active checks: implementation
name correspondence; arbitrary receiver model mutation and visibility inference; unknown tuple-returning/panicking
methods; temporal primitive intent; callback intent; and semantic rustdoc paraphrases. Exact parts of each mixed rule
remain active.

Runtime receiver inference includes supported Result success paths through match, map_err and question marks,
with resolved type aliases; an error type named Runtime does not establish a runtime success receiver.
Source traversal respects Cargo-resolved syn/quote literal token grammar while retaining span expressions,
interpolations and custom macro analysis. This does not expand arbitrary macros.
The direct question-mark composition check applies the general one-step-per-let clause to both Result and
Option; discarded-result exceptions remain a separate check.

### How the rules read a workspace

What the Rust documents leave to the checker, stated here. Each line was checked against the code.

- **Technology words.** A crate's technology words are `file`, `memory` and `system`, the full name of each of its
  external dependencies that is neither a workspace member nor a development dependency with `-` as `_`, that
  name's first word unless it is generic (`alloc`, `async`, `common`, `core`, `derive`, `lib`, `macros`, `rt`,
  `runtime`, `rust`, `std`, `sync`, `sys`, `util`, `utils`), and the words of `technology-words`, less `database`,
  `network`, `local` and `remote`. `async-trait` gives `async_trait` only; `aws-sdk-s3` gives `aws_sdk_s3` and `aws`.
- **Role detection order.** The rows of Roles are tried in their order, so `platform` comes before `technology`: a
  platform crate is named after the host framework it depends on, and its path matches that dependency too. While
  `platform-words` is empty no crate is detected as `platform` or `core-platform`, and such a crate declares its
  `role`.
- **Direction.** `KindImportDirection` compares kinds, not concepts: a row holds for two modules of any concepts of
  the crate. The direction between concepts is not checked beyond `PackageDependencyCycle`, which reports cycles
  among kind modules.
- **Cargo cycles.** Cargo rejects a cycle of normal dependencies and only warns about a dependency on a crate with no
  library; `UnitDependencyCycle` reports a cycle that closes through a development edge, and `EntryPointDependency`
  a dependency on an entry point or a member with no Rust library target.
- **`mod` in a file module.** `RootFileShape` reports every `mod`, inline or not, in a file other than the crate
  root: "only the crate root declares modules".
- **Fields and doc-hidden items.** `unreachable_pub` does not report a field, so `VisibilityTier` does. A
  `#[doc(hidden)] pub` item that no other workspace crate uses is `DocHiddenUse`.
- **The `event` kind.** `KindContent` reads names only: a struct, enum or union is allowed there when it is named
  `…Listener` or `…Event`, or is plain data (no hand-written `impl` holding a function in the production tree) in a
  file that declares a type so named. A helper renamed `…Listener` is not caught, and neither is state handed to
  listeners as a bare `Arc<Mutex<…>>`: both are the reviewer's.
- **Trait impls of a model.** `ModelContainsFunctions` reads inherent impls only, and `UtilFunctionAsExtension` does
  not examine trait impls: whether a trait impl of a model type holds a rule that belongs in `util` is the
  reviewer's call.
- **Generated files.** A file whose first line is a `//` or `/* */` comment containing `@generated` is skipped by the
  rules that read file contents, among them `OneTypeFamilyPerFile`, `UtilFileName`, `UtilShape`, `LocalItem`,
  `NamespaceType`, `DeclarationOrder`, `CalleeBeforeCaller`, `ConstantOutsideImpl`, `ImportOrdering`,
  `WildcardImport` and `ForbiddenComment`. The `mod` line that declares it is read as any other.

- **Macros and the name index.** The checks that prove "no other crate names this item" (`DiModuleVisibility`,
  `DocHiddenUse`, the workspace pass of `VisibilityTier`, the `unused_public` branch of `ReExportShape`, the `stub` and
  `memory` branch of `ImplementationVisibility`, the owner branches of `NestedDataStruct` and `NestedEnum`) stop while
  a name may be hidden. A macro in expression or statement position whose arguments read as comma-separated
  expressions (`warn!("{}", value)`, `write!(buffer, "x")`) hides none, since it adds no item to a module and its
  arguments are visited, and the names a format string captures inline (`info!("{LIMIT}")`) are uses; `include!`
  hides names in any position. A macro at item position, in an `impl` or a trait, with arguments that do not read that way,
  any `macro_rules!`, a generated file or an unresolved path hides names. In a crate with no Rust library target an
  item macro called through a non-workspace dependency with plain identifier arguments hides none either: no crate can name what it
  generates. The linter does not expand macros: it reads text on the pinned stable toolchain, and an expansion
  needs a compiler pass (`cargo expand` and `-Zunpretty=expanded` are nightly-only, and `notes.rs.md` keeps projects
  on stable), the `ra_ap_*` crates (rust-analyzer as a library, unversioned) or a `macro_rules!` expander of its own.

Not carried over from the documents: "the checker holds the direction between concepts" (it does not: see
Direction), and the comparisons with Kotlin's checker (how it takes a library's first word, that it skips the
generated sources under `build/`), which describe no Rust behaviour.

## Development

```text
rules-rust/
  Cargo.toml, clippy.toml, rust-toolchain.toml, .cargo/config.toml
  modules/rules/   the published crate `klibs-rules`: src/, test/ (case workspaces, both gate halves)
  xtask/           `cargo xtask gate-sync --docs <dir> [--check]`
  cases/           standalone workspaces, excluded from this one
    rules/<RuleId>/<fail|pass>-<case>/        expected.tsv
    suppression/<case>/                       expected.tsv and accepted.tsv
    configuration/<case>/                     expected-error.txt (exit 2)
    consumer/{failing,fixed,single-crate}/    the template
    doc/                                      the documents' examples, examples.toml, expected-scaffold.tsv
```

- `KLIBS_RULES_BLESS=1 cargo test --workspace` rewrites `expected.tsv`, `accepted.tsv`, `expected-error.txt` and
  `cases/doc/expected-scaffold.tsv`; the diff is reviewed like code.
- The gate (`cases/doc`) holds every Bad and Good example of the documents. `examples.toml` maps each half to its
  fragments (`block-lines` in the document, `fixture-lines` in the fixture); the tests check that the fixture lines
  hash to the document's, that Good halves have no rustc, clippy or rule finding, that Bad halves give exactly their
  carriers, that every probe of the required configuration behaves, and that rustfmt changes no Good or scaffold
  line. Findings outside the halves' lines are compared with `expected-scaffold.tsv`. With
  `KLIBS_RULES_EXPECT_TOOLCHAIN=<version>` the tests assert that `rustc` and `$CARGO` in `cases/doc` are that version.
- `cargo xtask gate-sync --docs <conf.ide>/.claude/docs` re-hashes the examples and records the `conf.ide` commit;
  `--check` exits 1 on a changed, moved, new or unplaced example.
- Validation tests prove that incorrect code is reported and correct code is accepted, including finding subjects,
  locations and documented exceptions. Coverage percentages do not block CI; tests are added for behavior and
  regressions rather than to exercise implementation lines or assertion failure paths.
- CI: `rust-pr.yml` (floor 1.93.0 through `RUSTUP_TOOLCHAIN`, and the pinned toolchain with the gate sync),
  `rust-release.yml`, `rust-msrv.yml` (weekly fresh resolution of
  `cases/consumer/fixed` on 1.93.0), `rust-doc-drift.yml` (weekly gate sync). None is a required check.
- Release: bump `version` in `modules/rules/Cargo.toml`; on merge `rust-release.yml` finds the commit that set the
  version, runs the gate sync on it, tags it `rules-rust-<version>`, publishes from the tag through crates.io Trusted
  Publishing (`rust-lang/crates-io-auth-action@v1`) and creates the GitHub release; every step resumes after a failure.
  When the documents drifted after the bump, the release stops before tagging: re-sync the gate and bump the version
  in one commit, which abandons the drifted version. The first release, 0.1.1, is published by hand from its tag
  with the owner's token read from the environment, after which the Trusted Publisher is registered.
