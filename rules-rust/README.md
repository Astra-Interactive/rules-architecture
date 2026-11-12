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
6. Pin the toolchain in `rust-toolchain.toml`: `LintConfig` asks the rustfmt beside the running cargo which keys are
   nightly-only, and that list differs by release (`hex_literal_case` is stable on 1.99.0, nightly-only on 1.93.0), so
   an unpinned project can get a rustfmt finding on one machine and none on another.

```toml
# xtask/Cargo.toml (no `version`: the crate is not published)
[package]
name = "xtask"
edition = "2024"

[dependencies]
klibs-rules = "=0.1.1"

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
- **Retired ids** (0.1.1): `ActualFileSuffix` → `PlatformFileSuffix`, `CallbackFlowShape` → `CallbackBridgeShape`,
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

## Rules in 0.1.1

| Id | Anchor | Reports |
|---|---|---|
| `LintConfig` | none | every missing or lowered entry of the required configuration (above) |
| `RootSources` | workspace | a `[package]` table in the root manifest; a `src/` at the root |
| `UnitReadme` | crate | a member without a `README.md` of its own (an inherited workspace `readme` does not stand in for it), a README whose sentence count is not one (counted as Kotlin counts), `readme` set to another path or `false` |
| `NestedUnit` | crate | a member directory nested inside another member |
| `ContainerSources` | workspace | a container holding a manifest, source directory or Rust file, or a child directory that holds files and has no member at or below it, or an immediate child directory with no member at or below it that the directory walk skips by name (`resources`, `build`, `node_modules`, a hidden directory) |
| `UnitPath` | crate | tooling outside `xtask`, entry points outside `instances/<name>`, other members outside `modules/`, a `shared` path segment, an unregistered `xtask` |
| `ProjectDirMatchesPath` | workspace | a wildcard in a workspace member path |
| `RootBuildSubprojects` | crate | literal shared package settings or dependencies instead of workspace inheritance (`foo = "1"` and `foo = { version = "1" }` are one requirement; a renamed dependency, `alias = { package = "foo", … }`, is compared with the root by its package name); profiles, patches or replacements in a member |
| `CrateTargets` | crate | custom library or binary roots, mixed library/binary targets, multiple binaries, `src/bin/` or a `test` target whose path is outside `tests/` |
| `UnitDependencyCycle` | none | a strongly connected component of normal, build and development edges containing a development edge |
| `EntryPointDependency` | none | any dependency on an entry point or a member without a Rust library target |
| `RootPackagePath` | crate, workspace | a name other than `<prefix>-<path without its container, / → ->`; `[lib] name` set; a crate named `std`, `core`, `alloc`, `proc_macro` or `test` in either spelling |
| `PublishedVersion` | crate | `version` (literal or inherited) with `publish = false` or `publish = []`, and `publish` true or a registry list without `version` |
| `XtaskAlias` | workspace | an `xtask` member without `[alias] xtask = "run --package xtask --"` in the workspace Cargo configuration (the array form `["run", "--package", "xtask", "--"]` is the same alias) |
| `ShellAutomation` | workspace | a `*.sh`, `*.bash`, `*.zsh`, `*.ksh`, `*.fish`, `*.csh`, `*.ps1`, `*.bat`, `*.cmd`, `Makefile`, `makefile`, `GNUmakefile`, `Makefile.toml`, `justfile`, `Justfile`, `.justfile`, `.Justfile`, `Taskfile.yml`, `Taskfile.yaml` or any other file (a dotted name included) whose shebang runs `sh`, `bash`, `zsh`, `dash`, `ash`, `ksh`, `fish`, `csh` or `tcsh`, at the workspace root or directly in a member directory; deeper in the tree, such a file that runs `cargo` (also after a shell keyword: `if`, `elif`, `while`, `until`, `then`, `do`, `else`, `!`, `{`, `}`). Approximate: what counts as build automation is the reviewer's call |
| `DependencyFloor` | workspace | `Cargo.lock` resolving a crate below the supported floor recorded by this library |
| `TimeCrate` | workspace | members that together declare more than one of `chrono`, `time` and `jiff`, including development dependencies and renamed packages |
| `PlatformSourceSetUnit` | crate | a member, other than an entry point, whose last path segment is a target word (documented resolution policy; `dragonfly`, `solaris`, `fuchsia` and `haiku` count only when the crate's own cfgs or target dependencies name them), as Kotlin reports a unit named after one of its eight targets (and not the root or an entry point) |
| `UnitDependency` | edge (manifest) | normal/build member edges forbidden by the dependent role, fake-feature boundary (a crate's own `fake = ["dep/fake"]` feature counts only when its `default`, or a `[dependencies]` edge of another crate that enables it, reaches the fake), tooling or proc-macro boundary |
| `ApiUnitDependency` | edge (manifest) | dependencies forbidden by the architecture's api role row |
| `TechnologyDependency` | edge (manifest) | a dependency on an external crate whose word is the word of a `technology` crate of the workspace, declared by a crate other than that `technology` crate and the entry points |
| `PluginImplUnit` | crate | under `product-kind = "platform-plugin"`, a crate of role `impl` or `impl-ui`, and a sibling of an `api` crate whose role is neither `platform` nor `api`; each `context`-role unit of a directory below the container that holds two or more `context` units and no `api`, `contract` or `storage-contract` unit (units of other roles in that directory are not reported by this check and do not count toward the two) |
| `ApiUnitPlatformDependency` | edge (manifest) | a platform crate (documented resolution policy) declared by a plugin's `api` anywhere, by an app's `api` outside `[target.'cfg(…)'.dependencies]`, or by `core-api`, `storage-contract` or `contract` anywhere |
| `PlatformDependencyTable` | edge (manifest) | a normal dependency declared outside `[target.'cfg(…)'.dependencies]` whose every import, path reference, macro-call path and attribute path in the dependent crate sits under a `cfg` predicate that implies a target atom (`all` if any child implies one, `any` only if every child does, `not` never) |
| `DocHiddenUse` | node | a `#[doc(hidden)] pub` item (including a constant, static or union) or a `pub` inherent impl or trait member that no other workspace crate names or implements; a published library crate is checked as well |
| `DiModuleVisibility` | node | a `pub` DI module that no other workspace crate names |
| `RootFileShape` | node, file | a root item outside the ordered module tree and platform items, modules in file modules, an invalid fixture block, a `test` block that is not private or an undeclared Rust file; proc-macro exports are allowed, and in an `entry-point` root a `static` named in `host-items` or carrying `#[no_mangle]` or `#[export_name]` is a platform item |
| `ModuleVisibility` | module | visible file modules or inline visibility inconsistent with reexports and test boundaries |
| `TestMirror` | node, module, crate | tests outside the mirrored test tree (a `cfg` that requires `test`, such as `cfg(all(test, unix))`, counts; `cfg(any(test, …))` does not), a test without a source owner or behaviour, a misplaced test block or path attribute; proc-macro expansion tests may use `tests/it/` |
| `ForbiddenPackageName` | module | a module named in `arch-package.md`'s list, `prelude` (a file module too), `composable`, `theme`, `viewmodel`; `std`, `core`, `alloc`; a raw identifier; `test` or `fixture` other than the two blocks; `bin` directly under `src/` |
| `PlatformPackage` | module | platform names used as modules in a crate with target predicates or target dependencies; the fixed word list holds only words with no ordinary meaning (`x86_64`, `aarch64`, `wasm32`, `uefi`, …), and `arm`, `none`, `hexagon`, `haiku`, `dragonfly`, `solaris`, `fuchsia`, … count only when the crate's own cfgs name them |
| `KindContent` | node | in the source tree, an item a kind does not hold: a function, `static`, struct, enum, union or `const` in `api`; a function or `static` in `model`; a function, `static`, enum, union or `const` in `usecase`; items under a target `cfg` (one file per target) are exempt |
| `ImplementationPlacement` | node | a struct or enum in a technology module that implements a workspace `api` trait of another concept |
| `ForeignErrorType` | node | a free function or inherent method in a `model`, `usecase` or `util` kind of the source tree whose `Result<T, E>` names a foreign `E` (`std::io::Error`, `Box<dyn std::error::Error>`); trait-impl methods keep the trait's signature |
| `ReExportShape` | node | missing/duplicate/renamed/glob kind re-exports, re-exported private items, a re-export of a kind block's item in any other block, an ancestor block included, public re-exports without external users, crate-private re-exports whose item has external users, and a `pub(super)`, `pub(self)` or `pub(in …)` re-export |
| `ImportThroughFileModule` | node | a `use`, type or expression path that walks through a file module, except the kind block's own re-exports |
| `ApiConfiguration` | node | `pub use` or `pub(crate) use` of a path whose first segment is an external crate |
| `KindShape` | module | a concept block holding a file module (at that file module's declaration); a block holding both files and blocks; a kind block inside a kind block; an empty kind block; a block that holds files and is no kind |
| `KindImportDirection` | node | imports forbidden by the architectural kind directions, including framework/registration boundaries, respecting cfg and test boundaries |
| `PackageDependencyCycle` | node | each strongly connected component of more than one kind module among production edges, under compatible cfg predicates, with one `file:line` per pair |
| `PlatformImport` | node | in `model`, `api` and `usecase` modules and in every module of a plugin `api`, `core-api`, `storage-contract` or `contract` crate, a path to a platform crate or a technology crate (a crate whose word names a technology module or `technology` crate of the workspace, or is in `technology-words`) |
| `ContractPlacement` | node | a trait outside `api`, except a `<Type>Factory`, a `<Type>Ext` in `util`, a use-case trait in `usecase` (P3: app `api` crates only), a state holder's trait beside its `Default…` in an entry-point kind, and a registration trait in its mechanism's module; an `impl` of a crate trait inside `api`; a `stub` module holding anything but structs that implement crate traits. P3: an app use-case trait not in the owner's `api` `usecase`, or its `Default…` not in the `impl` `usecase`. Approx registration shape: every method synchronous, required, returning `()`, taking `&self` plus parameters whose types all come from external crates; such a trait may sit in an entry-point kind or in mixed `core`, but not in `model`, `usecase` or `util` of a core crate |
| `UseCaseShape` | node | placement/naming/public-surface checks, use-case chaining, supported plain contract passthrough and contract field ownership; semantic business-rule necessity remains review |
| `UseCaseAbstraction` | node | a trait named `*UseCase` outside the `usecase` kind of an app's `api` crate |
| `UtilShape` | node | a struct, enum, union, type alias, const or static in `util`, and an inherent `impl T` outside T's file and outside `util`, except in `extension-kinds` |
| `UtilFileName` | node, module | generic utils/helpers/extensions filenames and supported util receiver/predicate naming mismatches, where the owner of a `<Type>Ext` may be a primitive, slice, array or reference type; generated files skipped |
| `UtilFunctionAsExtension` | node | a free function in `util` with exactly one parameter of a crate model type, `&`, `&mut`, `Box`, `Rc` and `Arc` removed (`Vec` and `Option` are not removed); a `create_*` returning `Result<Model, _>` is the aggregate factory and passes |
| `PluralPackageName` | module | a block name ending in `s` but not in `ss`, `us` or `is`, unless in `plural-concepts` or one of Kotlin's two built-in words, `news` and `settings` (README); the singular words `alias`, `atlas`, `bias`, `canvas`, `christmas`, `gas` and `pancreas` are not plural (a closed list: another singular word in `-as` goes into `plural-concepts`) |
| `DiPlacement` | node, module | misplaced wiring structs, missing DI for proven dependency assembly/resource ownership, and DI in proven contracts/models-only crates |
| `RootModulePlacement` | node | a `root-module` struct outside `src/di/root_module.rs` of an `entry-point` crate, and more than one in a crate |
| `EntryPointShape` | node, module | in an `entry-point` root, blocks other than `di`, `model`, `error` (the framework failure type's kinds) and the fixture and test blocks; `fn main` with any visibility modifier; a non-`pub` host item, or function or `static` with `#[no_mangle]`, `#[unsafe(no_mangle)]` or `#[export_name]`, in a cdylib or staticlib |
| `EntryPointVisibility` | node | a `pub` type in an entry-point kind, except an app `api` state holder's interface and a `pub` event type in the `event` kind of a `platform` crate that carries a host derive or trait: a derive or trait of a foreign, non-workspace crate, or, when `platform-words` is set, only one that resolves to a platform identity (the same test as `EventPlacement`) |
| `ImplementationVisibility` | node | a `pub` struct or enum in a technology module that implements a crate trait, unless the module is `stub` or `memory` |
| `CoreShape` | node, module | a `usecase` kind, a `*Repository` trait, a `*Repository` struct or enum or an `impl` of a `*Repository` trait (the last two in `src` only), in a `core`, `core-api`, `core-platform` or `core-ui` crate |
| `ContractUnitShape` | node | in a `contract` crate, a struct or enum lacking `Serialize` or `Deserialize`, a type alias, a union, a `static`, and any function body; consts are route constants and allowed |
| `ErrorPlacement` | node | an enum or struct deriving `Error`, holding a hand-written `impl` of `std::error::Error` or `core::error::Error`, or named `*Error`, or an enum named `*Result`, or a crate enum that a function or method of the crate returns as the `E` of `Result<T, E>`, outside `model`, one in an old `error`, `errors` or `exception` module included; in an entry-point crate, a workspace error enum outside the crate's `model` returned by a `route` function or inherent method |
| `EnumFamilyFile` | node | a struct used only as a payload of one enum's variants but declared outside that enum's file |
| `NestedDataStruct` | node | a data struct used as a field type by exactly one type but not in its file, and a type returned to another type but not in its own file in `model`; a type declared in the file of its single field owner that a free function of that file, another type's `impl` or a use outside the owner also names; name counts from the crate index |
| `NestedEnum` | node | an enum used only through one type but declared outside its file; an enum declared in the file of its single field owner that a free function of that file, another type's `impl` or a use outside the owner also names |
| `FakePlacement` | node, crate | separate fake crates, incorrect fake tree/concept/visibility, missing fixture feature pairing, `fake` in the `default` feature list or reached from it through other features of the crate, a fake of another workspace crate's trait in the test tree, or duplicate test/fixture fakes |
| `InMemoryPlacement` | node | an `InMemory*` struct or enum outside a `memory` module, and an implementation of trait `T` in `memory` not named `InMemory<T>` |
| `ImplPlacement` | node | an `impl` of a crate type outside that type's file, except: an inherent impl in `util/<t>_ext.rs`; `From` or `TryFrom` between a model type and a type declared in the file holding the impl (the technology representation's file); `From` or `TryFrom` between a model type and an external crate's type, in a technology module of that crate; P3: a framework's trait for an error enum, in the `error` kind of an `entry-point` crate |
| `UnsafePlacement` | node | an `unsafe` block, `unsafe fn`, `unsafe impl`, `unsafe trait`, `unsafe extern` block, or an `#[unsafe(…)]` attribute (`no_mangle`, `export_name`, `link_section`) outside a technology module, except the attributes in the root of an entry-point crate |
| `ListenerName` | node | a struct, enum or union implementing an external `*Listener` trait that is not named `*Listener`, or implements that trait outside `event`; unrelated external traits such as `Drop` do not establish listener intent |
| `EventPlacement` | node | a type named `*Event` outside `model`; in the `event` kind of a `platform` crate a type that implements a trait of another crate, by an `impl` or by a `#[derive(…)]` that resolves to a crate other than `std`, is the platform's event and passes; when `platform-words` is set, only an `impl` or a derive that resolves to a platform identity (a `platform` or `core-platform` workspace crate, or a crate named by a platform word) exempts it, so a `serde` derive or another crate's trait no longer does, and a host trait from a crate outside the platform words needs that word added |
| `IntraDocLink` | node | an inline code span in a doc comment holding exactly an identifier (or `Type::member`), with or without a trailing `()`, that resolves in the item's scope and is not written as an intra-doc link |
| `VisibilityTier` | node | public reachability and indexed type/item users; `pub(super)`, `pub(self)` and `pub(in path)` on items, fields and methods; in an unpublished crate, a `pub` field or method of an externally used type whose name no other workspace crate uses (a tuple field counts as used when another crate writes its index after a dot, or the type's name, or the name of a `type` alias or `use … as` rename of the type, before a parenthesis, and inside a crate a `pub(crate)` tuple field counts as used when another file of the crate writes the name of a `type` alias or `use … as` rename of its type before a parenthesis); the users it indexes include the handlers of a `select!` macro; inferred widest visibility for fields/methods remains review |
| `AssociatedFactoryFunction` | node | an associated function without `self` on a model type other than `new`, wherever its `impl` is; a free `create_*` returning a proven model, directly or through supported Result/Option aliases, outside `util` or without `Result`; naming alone does not classify a helper as an aggregate factory |
| `ModelContainsFunctions` | node | for a model type: more than one inherent `impl` in its file; in that `impl`, anything but associated consts and, when `new` calls `assert!`, `new` and accessors (one field read of `self`, also through argument-less `as_*`, `clone`, `copied` or `cloned` calls); any other method in the type's file |
| `ModelShouldBeImmutable` | node | mutable self receivers (`self: Pin<&mut Self>` included) and field writes through supported typed model parameters, locals and reference aliases; arbitrary receiver inference remains review |
| `ForbiddenImport` | node | a path to `chrono`, `time` or `jiff` outside technology modules, `technology` crates and `contract` crates |
| `ForbiddenComment` | none | region markers (`region`/`endregion` followed by the end, whitespace or `:`, with an optional `#` or `pragma`; `<editor-fold`, `</editor-fold>`, `MARK:`), separator lines and section headers, framed with `=`, `-`, `*`, `#`, `~`, `_`, `+`, `/` or a repeated non-ASCII symbol, or one-sided (`=`, `-`, `*` in doc comments; also `#`, `~`, `_`, `+` and a repeated non-ASCII symbol in other comments), in comments and doc comments (fences, thematic breaks in doc comments and Markdown emphasis runs aside); generated files skipped |
| `ClosureParameterName` | node | a one-letter closure parameter, including destructured bindings |
| `ExplicitItName` | node | an `it` binding, parameter, field, loop label, owned item or import binding, also inside the branches of a `select!` or `select_biased!` macro (tokio's and futures' grammar, and crossbeam's `recv(r) -> x`, `send(s, v) -> x` and `default(timeout)` branches, read for a bare `select!` or `select_biased!` and for the `tokio`, `futures`, `crossbeam` and `crossbeam_channel` paths only); trait-required names are preserved |
| `FunctionVerb` | node | a function starting with `get_`, `map`, `make`, `do`, `handle` or `process`; receiverless collection lookup names also report; trait methods and tests keep their names, as do `get`, `get_mut`, `get_unchecked` and `get_unchecked_mut` taking a receiver and a key parameter, and a method whose body is only its own field read (`self.<name>`, `&self.<name>`, `&mut self.<name>` or `self.<name>.clone()`, `<name>` being the method name) |
| `ImplementationName` | node | Impl/Interface suffixes, an `Impl` prefix (`ImplRatingRepository`; not `Implicit…`), interface I prefixes, and generic Database/Network/Local/Remote prefixes (ending at a word boundary) on proven implementations, those of a use-case trait in the `usecase` kind included, and on type aliases; correspondence with a full trait name remains review |
| `RolelessTypeName` | node | as Kotlin does: a struct, enum, union or trait named `*Validator`; one named `*Factory` that is not the runtime-factory form; one named `*WarmUp` outside a `warmup` module and `*Migration` outside a `migration` module |
| `KindNameInPackage` | node | utility suffixes, Model/Entity repetition in model kinds, API suffixes on contracts, and the word Service in any type name outside a `service` module (module path only, not the crate name); a type alias is checked like a declared type |
| `BareIdentifier` | node | bare standard strings, UUIDs or integer primitives (including supported aliases and the wrappers `Option`, `Box`, `Rc`, `Arc`, `Cow<str>` and `NonZero*`) in ID fields, in an `ids` or `*_ids` field holding a `Vec`, `VecDeque`, `HashSet` or `BTreeSet` of a bare type, malformed typed-ID structs, a `type <Name>Id = <primitive>` alias in `model`, and single-property named model value objects (a one-field struct whose field is `id: <…>Id` is an entity and passes); serialization alone does not exempt domain models, while technology/wire contract boundaries do |
| `TestFunctionName` | node | a test name outside `given_<state>_when_<action>_then_<outcome>`; a test attribute counts when it resolves to `tokio::test` through a `use` |
| `CallbackDeclaration` | node | outside the test and fixture trees, as Kotlin does outside test sources: a function parameter or a struct field named `on_*` whose type, bare or inside `Option` or another single-value wrapper (`Mutex`, `RwLock`, `OnceLock`, `Cell`, `RefCell`, `OnceCell`), is a closure type (`impl Fn…`, `dyn Fn…` behind any pointer, an `fn(…)` pointer: Kotlin's function type) or a type parameter bounded by `Fn`, `FnMut` or `FnOnce`; a trait the project declares named `*Listener` or `*Callback` |
| `DeclarationOrder` | node | constants, type families, constructors or trait associated items out of their required order; a constant with a value inside a trait body (it belongs at the top of the file) |
| `CalleeBeforeCaller` | node | free-function callers above callees in files without type declarations, and own-type method ordering; includes self-qualified paths and proven local function aliases with lexical scope; constructors and mutual recursion are exempt; arbitrary dynamic call targets remain review |
| `OneTypeFamilyPerFile` | node, module | a mismatched file name, multiple behavioural types, unrelated secondary types (a type's reference to itself is no use) or free functions beside a type, a secondary struct or enum named `*Error` or deriving `Error` included; `<Type>Factory` is exempt |
| `ConstantOutsideImpl` | node | a type's constants outside or before the end of its inherent impl; a file module holding only constants beside the file of its owner type (the module name is the owner's snake-case name followed by `_` and a word, or the owner's file mentions the module) |
| `NamespaceType` | node | a struct without fields, or an enum without variants, with an inherent impl holding a function and no trait impl |
| `LocalItem` | node | `struct`, `enum`, `union`, `trait`, `impl` or `mod` inside a function body, closure or `async` block |
| `StructDefaultValue` | node | project defaults and omitted own-type fields through struct update syntax, also inside the handlers of a `select!` macro, with documented deserialization/constructor exceptions |
| `AnonymousTuple` | node | stored multi-element tuple types/structs/variants and proven tuple-returning APIs, including supported enumerate and `char_indices` elements, `iter`, `iter_mut` and `into_iter` over a typed std `HashMap` or `BTreeMap`, tuples built by `if`, `match` and block initialisers, scoped local iterator aliases and receiver-specific standard partition/unzip results with workspace extension-override guards; immediate local destructuring, including its type annotation on a `let` or on a closure parameter, passes; the handlers of a `select!` macro are read; unknown receiver return types remain review |
| `WildcardImport` | node | every `use` tree with a glob |
| `PlatformFileSuffix` | node | a `mod x;` under a single-key target predicate (documented resolution policy) whose name does not end in `_<value>`; a compound target predicate (`all`, `any`, `not`) on a `mod`; platform files whose re-exports differ in name or are not under the same predicate; a target predicate on anything other than such a `mod` and its re-export |
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
| `ModelDerives` | node | a model struct without `Debug`, `Clone` and `PartialEq`, a `*Id` without `Debug, Clone, PartialEq, Eq, Hash`, and missing `Copy` when its inner type is proven Copy; a struct whose derive resolves to, or is written as, a derive named `Error`, or that implements `Error`, is skipped |
| `ValueObjectShape` | node | validated-model field privacy/accessors (a field read, also through argument-less `as_*`, `clone`, `copied` or `cloned` calls) and constructor purity, plus supported parse-function location, signature (the error argument of its `Result`, after alias expansion, is a workspace enum) and single-construction flow |
| `ContractShape` | node | contract supertraits (`Send`, `Sync` and `'static` pass, any other is reported; generics and where clauses are not reported), missing async_trait, non-BoxStream stream contracts, a returned future type (`BoxFuture`, `LocalBoxFuture`, `impl Future`, `Pin<Box<dyn Future>>`), a returned `tokio_stream::wrappers` stream, a returned `std::sync::mpsc` or `futures` `mpsc` receiver, a `BoxStream` whose first lifetime is not `'static` under any method name, a tokio `mpsc::Sender`, `mpsc::UnboundedSender`, `broadcast::Sender` or `watch::Sender` parameter or a `std::sync::mpsc` `Sender` or `SyncSender` parameter (`oneshot` passes), callback/task handles and external error types (a one-argument `Result<T>` resolved to an external alias included) |
| `ErrorFamilyShape` | node | non-enum error families, missing Debug/thiserror::Error/error messages, non-boxed foreign source fields (a `#[source]` of a workspace family passes), proven foreign from/source payloads (an external type named `*Error` or with an `error` module in its path, such as `tokio::time::error::Elapsed`), and fields holding `Rc`, `Weak`, `Cell` or `RefCell` at any depth |
| `ErrorLogging` | node | supported caught errors logged without an error-chain field, logged repeatedly on the same supported path, or logged before propagation; arbitrary logger adapters and interprocedural exactly-once handling remain review |
| `CatchUnwind` | node | `catch_unwind` whose closure calls a crate-local item, or whose argument (also inside `AssertUnwindSafe`) is a path to a crate-local function or to a local closure that calls one, through a chain of local closures each followed once; a local closure is not taken for a crate-local function of the same name |
| `DocCommentScope` | node | mechanically redundant application/trait-implementation docs, malformed contract headings, missing published error/safety contracts and proven panic docs (`unwrap`, `expect`, `unwrap_err` and `expect_err` on a parameter; `panic!`, the assertion macros, `unreachable!`, `todo!` and `unimplemented!`), including supported caller-controlled indexing and default trait bodies; external reasons, additional contract information and unknown panic behavior remain review |
| `GlobalState` | node | a `static` typed `OnceLock`, `LazyLock`, `Mutex` or `RwLock`, also inside std `thread_local!`; `lazy_static!`; installing a logging subscriber (`set_global_default`, `.init()` on a subscriber builder) outside `fn main` |
| `DiModuleShape` | node | DI declarations outside the permitted struct/trait forms, service construction outside new, generic contract parameters (on the impl, or on an inherent `fn` through its generics, where clause or `impl Trait` parameter, bounded by a workspace trait; on a use case every inherent `fn` but `new`, which `UseCaseShape` reads), unsupported sharing/deferred storage and contract injection despite an accessible compatible provider module; a DI trait's contract trigger and external semantics remain review |
| `ConstructorDependency` | node | supported construction/factory calls of proven replaceable dependencies outside composition boundaries, a free function included unless it sits in `di` or in the module that declares the constructed type; data containers and host root assembly are preserved |
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
| `Unanalysable` | node, crate | a root file or file module that does not parse, a missing module file, `include!`, `cfg_attr(…, path = …)`, an item macro call in the root file |
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
