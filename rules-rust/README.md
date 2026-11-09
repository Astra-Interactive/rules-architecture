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
klibs-rules = "=0.1.0"

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
- **Retired ids** (0.1.0): `ActualFileSuffix` → `PlatformFileSuffix`, `CallbackFlowShape` → `CallbackBridgeShape`,
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
repository's root uses them. `LintConfig` reports a required lint that is missing or lowered (`allow`, `expect`, a
group set to `allow` with a priority not below the lint's own), a `check-cfg` without `cfg(klibs_rules)`, a missing or
false `clippy.toml` key, a missing threshold, a missing `disallowed-*` or `await-holding-invalid-types` path,
`allow-print-in-tests` or `allow-dbg-in-tests`, a `clippy.toml` in a member directory or two at the root, a member
without `[lints] workspace = true`, a root without `resolver = "3"` (or the Cargo configuration's
`incompatible-rust-versions = "fallback"`), a toolchain file pinning `nightly`, `beta` or a stable below 1.93.0, a
nightly-only `rustfmt.toml` key for the running rustfmt, `--cfg klibs_rules`, `--cap-lints`, `-A` or `--allow` in
`rustflags`, an `[unstable]` table and `cargo-features`. Extra lints are allowed: the project's configuration wins.

Lints that fail a Good example or a kept layout, and so are advice against, not a check: `arbitrary_source_item_ordering`,
`absolute_paths`, `tests_outside_test_module`, `min_ident_chars`, `clone_on_ref_ptr`, `renamed_function_params`,
`module_name_repetitions`, `redundant_pub_crate`, `pub_use`, `missing_docs`, `missing_errors_doc`, `missing_panics_doc`,
and the groups `clippy::restriction`, `clippy::pedantic` and `clippy::nursery` as a whole.

The floor is 1.93.0; the library declares `rust-version = "1.93"`.

## Rules in 0.1.0

| Id | Anchor | Reports |
|---|---|---|
| `LintConfig` | none | every missing or lowered entry of the required configuration (above) |
| `RootSources` | workspace | a `[package]` table in the root manifest; a `src/` at the root |
| `UnitReadme` | crate | a member without `README.md`, a README whose sentence count is not one (counted as Kotlin counts), `readme` set to another path or `false` |
| `NestedUnit` | crate | a member directory nested inside another member |
| `ContainerSources` | workspace | a container holding a manifest, source directory or Rust file |
| `UnitPath` | crate | tooling outside `xtask`, entry points outside `instances/<name>`, other members outside `modules/`, a `shared` path segment, an unregistered `xtask` |
| `ProjectDirMatchesPath` | workspace | a wildcard in a workspace member path |
| `RootBuildSubprojects` | crate | literal shared package settings or dependencies instead of workspace inheritance; profiles, patches or replacements in a member |
| `CrateTargets` | crate | custom library or binary roots, mixed library/binary targets, multiple binaries or `src/bin/` |
| `UnitDependencyCycle` | none | a strongly connected component of normal, build and development edges containing a development edge |
| `EntryPointDependency` | none | any dependency on an entry point or a member without a Rust library target |
| `RootPackagePath` | crate, workspace | a name other than `<prefix>-<path without its container, / → ->`; `[lib] name` set; a crate named `std`, `core`, `alloc`, `proc_macro` or `test` in either spelling |
| `PublishedVersion` | crate | `version` (literal or inherited) with `publish = false` or `publish = []`, and `publish` true or a registry list without `version` |
| `XtaskAlias` | workspace | an `xtask` member without `[alias] xtask = "run --package xtask --"` in the workspace Cargo configuration |
| `ShellAutomation` | workspace | a `*.sh`, `Makefile`, `makefile`, `justfile` or `Justfile` at the workspace root or directly in a member directory. Approximate: what counts as build automation is the reviewer's call |
| `DependencyFloor` | workspace | `Cargo.lock` resolving a crate below the supported floor recorded by this library |
| `TimeCrate` | workspace | members that together declare more than one of `chrono`, `time` and `jiff` as non-dev dependencies |
| `PlatformSourceSetUnit` | crate | a member, other than an entry point, whose last path segment is a target word (documented resolution policy), as Kotlin reports a unit named after one of its eight targets (and not the root or an entry point) |
| `UnitDependency` | edge (manifest) | normal/build member edges forbidden by the dependent role, fake-feature boundary, tooling or proc-macro boundary |
| `ApiUnitDependency` | edge (manifest) | dependencies forbidden by the architecture's api role row |
| `TechnologyDependency` | edge (manifest) | a dependency on an external crate whose word is the word of a `technology` crate of the workspace, declared by a crate other than that `technology` crate and the entry points |
| `PluginImplUnit` | crate | under `product-kind = "platform-plugin"`, a crate of role `impl` or `impl-ui`, and a sibling of an `api` crate whose role is neither `platform` nor `api` |
| `ApiUnitPlatformDependency` | edge (manifest) | a platform crate (documented resolution policy) declared by a plugin's `api` anywhere, by an app's `api` outside `[target.'cfg(…)'.dependencies]`, or by `core-api`, `storage-contract` or `contract` anywhere |
| `DocHiddenUse` | node | a `#[doc(hidden)] pub` item that no other workspace crate names |
| `DiModuleVisibility` | node | a `pub` DI module that no other workspace crate names |
| `RootFileShape` | node, file | a root item outside the ordered module tree and platform items, modules in file modules, an invalid fixture block or an undeclared Rust file; proc-macro exports are allowed |
| `ModuleVisibility` | module | visible file modules or inline visibility inconsistent with reexports and test boundaries |
| `TestMirror` | node, module, crate | tests outside the mirrored test tree, a test without a source owner or behaviour, a misplaced test block or path attribute; proc-macro expansion tests may use `tests/it/` |
| `ForbiddenPackageName` | module | a module named in `arch-package.md`'s list, `prelude`, `composable`, `theme`, `viewmodel`; `std`, `core`, `alloc`; a raw identifier; `test` or `fixture` other than the two blocks; `bin` directly under `src/` |
| `PlatformPackage` | module | platform names used as modules in a crate with target predicates or target dependencies |
| `ReExportShape` | node | missing/duplicate/renamed/glob kind re-exports, re-exported private items, public re-exports without external users, and crate-private re-exports whose item has external users |
| `ImportThroughFileModule` | node | a `use`, type or expression path that walks through a file module, except the kind block's own re-exports |
| `ApiConfiguration` | node | `pub use` or `pub(crate) use` of a path whose first segment is an external crate |
| `KindShape` | module | a concept block holding a file module (at that file module's declaration); a block holding both files and blocks; a kind block inside a kind block; an empty kind block; a block that holds files and is no kind |
| `KindImportDirection` | node | imports forbidden by the architectural kind directions, including framework/registration boundaries, respecting cfg and test boundaries |
| `PackageDependencyCycle` | node | each strongly connected component of more than one kind module among production edges, under compatible cfg predicates, with one `file:line` per pair |
| `PlatformImport` | node | in `model`, `api` and `usecase` modules and in every module of a plugin `api`, `core-api`, `storage-contract` or `contract` crate, a path to a platform crate or a technology crate (a crate whose word names a technology module or `technology` crate of the workspace, or is in `technology-words`) |
| `ContractPlacement` | node | a trait outside `api`, except a `<Type>Factory`, a `<Type>Ext` in `util`, a use-case trait in `usecase` (P3: app `api` crates only), a state holder's trait beside its `Default…` in an entry-point kind, and a registration trait in its mechanism's module; an `impl` of a crate trait inside `api`; a `stub` module holding anything but structs that implement crate traits. P3: an app use-case trait not in the owner's `api` `usecase`, or its `Default…` not in the `impl` `usecase`. Approx registration shape: every method synchronous, required, returning `()`, taking `&self` plus parameters whose types all come from external crates; such a trait may sit in an entry-point kind or in mixed `core` |
| `UseCaseShape` | node | a `*UseCase` struct outside `usecase`; a struct in `usecase` not named two or more words then `UseCase`; a public method other than `new` and `invoke`; no `invoke`; an `async fn invoke` without `.await`; a field mentioning a `*UseCase` type, or `.invoke(` on one |
| `UseCaseAbstraction` | node | a trait named `*UseCase` outside the `usecase` kind of an app's `api` crate |
| `UtilShape` | node | a struct, enum, union, type alias, const or static in `util`, and an inherent `impl T` outside T's file and outside `util`, except in `extension-kinds` |
| `UtilFileName` | node, module | a file named `utils.rs`, `helpers.rs` or `extensions.rs`; generated files skipped |
| `UtilFunctionAsExtension` | node | a free function in `util` with exactly one parameter of a crate model type, `&` and `&mut` removed |
| `PluralPackageName` | module | a block name ending in `s` but not in `ss`, `us` or `is`, unless in `plural-concepts` or one of Kotlin's two built-in words, `news` and `settings` (README) |
| `DiPlacement` | node, module | a `*Module` struct outside `di` |
| `RootModulePlacement` | node | a `root-module` struct outside `src/di/root_module.rs` of an `entry-point` crate, and more than one in a crate |
| `EntryPointShape` | node, module | in an `entry-point` root, blocks other than `di` and the fixture and test blocks; `pub fn main`; a non-`pub` host item in a cdylib |
| `EntryPointVisibility` | node | a `pub` type in an entry-point kind |
| `ImplementationVisibility` | node | a `pub` struct in a technology module that implements a crate trait, unless the module is `stub` or `memory` |
| `CoreShape` | node, module | a `usecase` kind, or a `*Repository` trait, in a `core`, `core-api`, `core-platform` or `core-ui` crate |
| `ContractUnitShape` | node | in a `contract` crate, a struct or enum lacking `Serialize` or `Deserialize`, and any function body; consts are route constants and allowed |
| `ErrorPlacement` | node | an enum or struct deriving `Error` or named `*Error`, or an enum named `*Result`, outside `model`, one in an old `error`, `errors` or `exception` module included |
| `EnumFamilyFile` | node | a struct used only as a payload of one enum's variants but declared outside that enum's file |
| `NestedDataStruct` | node | a data struct used as a field type by exactly one type but not in its file, and a type returned to another type but not in its own file in `model`; name counts from the crate index |
| `NestedEnum` | node | an enum used only through one type but declared outside its file |
| `FakePlacement` | node, crate | separate fake crates, incorrect fake tree/concept/visibility, missing fixture feature pairing, or duplicate test/fixture fakes |
| `InMemoryPlacement` | node | an `InMemory*` struct outside a `memory` module, and an implementation of trait `T` in `memory` not named `InMemory<T>` |
| `ImplPlacement` | node | an `impl` of a crate type outside that type's file, except: an inherent impl in `util/<t>_ext.rs`; `From` or `TryFrom` between a model type and a type declared in the file holding the impl (the technology representation's file); `From` or `TryFrom` between a model type and an external crate's type, in a technology module of that crate; P3: a framework's trait for an error enum, in the `error` kind of an `entry-point` crate |
| `UnsafePlacement` | node | an `unsafe` block, `unsafe fn`, `unsafe impl` or `unsafe trait` outside a technology module |
| `ListenerName` | node | a struct in `event` that implements an external trait but is not named `*Listener`, or is named `*Event`; an implementation of an external trait named `*Listener` outside `event` |
| `EventPlacement` | node | a type named `*Event` outside `model` |
| `IntraDocLink` | node | an inline code span in a doc comment holding exactly an identifier (or `Type::member`) that resolves in the item's scope and is not written as an intra-doc link |
| `VisibilityTier` | node | public reachability and indexed type/item users; inferred widest visibility for fields/methods remains review |
| `AssociatedFactoryFunction` | node | an associated function without `self` on a model type other than `new`, wherever its `impl` is; a free `create_*` function outside `util`, or one that does not return `Result` |
| `ModelContainsFunctions` | node | for a model type: more than one inherent `impl` in its file; in that `impl`, anything but associated consts and, when `new` calls `assert!`, `new` and accessors (one field read of `self`); any other method in the type's file |
| `ModelShouldBeImmutable` | node | mutable self receivers on model methods; arbitrary receiver field-assignment inference remains review |
| `ForbiddenImport` | node | a path to `chrono`, `time` or `jiff` outside technology modules, `technology` crates and `contract` crates |
| `ForbiddenComment` | none | region markers, separator lines and section headers in comments and doc comments (fences and thematic breaks in doc comments aside); generated files skipped |
| `ClosureParameterName` | node | a one-letter closure parameter, including destructured bindings |
| `ExplicitItName` | node | an `it` binding, parameter, field or loop label |
| `FunctionVerb` | node | a function starting with `get_`, `map`, `make`, `do`, `handle` or `process`; trait methods, tests and the four collection lookup names keep their names |
| `ImplementationName` | node | Impl/Interface suffixes, interface I prefixes, and generic Database/Network/Local/Remote prefixes on proven implementations; correspondence with a full trait name remains review |
| `RolelessTypeName` | node | as Kotlin does: a struct, enum, union or trait named `*Validator`; one named `*Factory` that is not the runtime-factory form; one named `*WarmUp` outside a `warmup` module and `*Migration` outside a `migration` module |
| `KindNameInPackage` | node | utility suffixes, Model/Entity repetition in model kinds, and Service/API suffixes on contracts |
| `BareIdentifier` | node | as Kotlin does: a field of a struct or of an enum variant named `id` or ending in `_id` (Kotlin's `id\|.*[a-z0-9]Id` in snake case) whose type, inside `Option` or not, is `String`, `&str`, `Box<str>`, `Arc<str>`, an integer primitive (Kotlin's `Int` and `Long`) or a path whose last segment is `Uuid`; and a struct named `*Id` that is not a tuple struct of one field (Kotlin's value class; `arch-domain.rs.md`, Typed identifier). Not, as Kotlin: a type deriving `Serialize` or `Deserialize` (Kotlin skips a `@Serializable` class); a type in a technology module or in a crate whose last path segment is `contract` (Kotlin skips technology and `contract` packages); an enum or trait named `*Id`. Approx (Id:4): a model struct with exactly one named field, which a single-property value object writes as a tuple struct |
| `TestFunctionName` | node | a test name outside `given_<state>_when_<action>_then_<outcome>` |
| `CallbackDeclaration` | node | outside the test and fixture trees, as Kotlin does outside test sources: a function parameter or a struct field named `on_*` whose type, inside `Option` or not, is a closure type (`impl Fn…`, `dyn Fn…` behind any pointer, an `fn(…)` pointer: Kotlin's function type) or a type parameter bounded by `Fn`, `FnMut` or `FnOnce`; a trait the project declares named `*Listener` or `*Callback` |
| `DeclarationOrder` | node | constants, type families, constructors or trait associated items out of their required order |
| `CalleeBeforeCaller` | node | a caller above its callee in one file or impl; constructors and mutual recursion exempt |
| `OneTypeFamilyPerFile` | node, module | a mismatched file name, multiple behavioural types, unrelated secondary types or free functions beside a type; `<Type>Factory` is exempt |
| `ConstantOutsideImpl` | node | a type's constants outside or before the end of its inherent impl |
| `NamespaceType` | node | a struct without fields, or an enum without variants, with an inherent impl holding a function and no trait impl |
| `LocalItem` | node | `struct`, `enum`, `union`, `trait`, `impl` or `mod` inside a function body, closure or `async` block |
| `StructDefaultValue` | node | project defaults and omitted own-type fields through struct update syntax, with documented deserialization/constructor exceptions |
| `AnonymousTuple` | node | multi-element tuple types/structs/variants and proven tuple-returning APIs; unknown receiver return types remain review |
| `WildcardImport` | node | every `use` tree with a glob |
| `PlatformFileSuffix` | node | a `mod x;` under a single-key target predicate (documented resolution policy) whose name does not end in `_<value>`; a compound target predicate (`all`, `any`, `not`) on a `mod`; platform files whose re-exports differ in name or are not under the same predicate; a target predicate on anything other than such a `mod` and its re-export |
| `RustdocHtmlTag` | node | an HTML tag `<tag …>` or `</tag>` in a doc comment, outside code spans and fenced blocks |
| `RustdocFiller` | node | fixed filler phrases and lexical restatement of parameter names; semantic paraphrases remain review |
| `BooleanOperatorWrapping` | node | an `if`, `while` or match-guard condition spanning several lines with a top-level `&&` or `\|\|` operand that is not a path, `!path` or literal |
| `ClosureSingleIfGuard` | node | a closure or `async` block whose whole body is one `if` without `else` around two or more statements |
| `OperatorBeforeTrailingCall` | node | a method call on a parenthesised arithmetic expression (one operator: write the `std::ops` method; two or more: name a `let`), and two chained calls of `add`, `sub`, `mul`, `div` or `rem` |
| `TrivialClosurePath` | node | chains of at least three path-only adapters that need named semantic steps; single function references remain allowed |
| `ImportOrdering` | node | use declarations after other items or separated into multiple blocks |
| `BlockOnOutsideBoundary` | node | `#[tokio::main]`; `block_on`, `Runtime::new` and `Builder::new_*` outside `fn main` of an `entry-point` root, except `block_on` in a technology module |
| `BlockOnInTest` | node | `block_on`, `Runtime::new`, `Builder::new_*` and `futures::executor::block_on` in the test and fixture trees |
| `IgnoredTest` | node | `#[ignore]` |
| `ExpectScope` | node | a lint expectation outside the prescribed lint list or smallest item, including inner and module-level expectations |
| `DetachedTask` | node | `tokio::spawn`, resolved through `use` aliases, as a bare statement, under `let _ =`, or inside `drop(…)` |
| `WatchTimeout` | node | a `timeout` call whose future is `<x>.wait_for(<a closure returning true>)`, or an `async` block without `.await` |
| `ImpureUpdateClosure` | node | a `fetch_update` closure that assigns a captured name |
| `CallbackBridgeShape` | node | spawning inside an argument closure; classifying external closures as asynchronous callbacks remains review |
| `BridgeCleanup` | node | `tokio::spawn` inside `fn drop` of an `impl Drop` |
| `ThreadPrimitive` | node | an `#[expect(clippy::disallowed_methods)]` covering a thread start outside a technology module |
| `DurationOverloadCall` | node | `Duration::from_*(<x>.as_millis() \| as_secs() \| as_micros() \| as_nanos() \| as_secs_f64() …)` round trips |
| `ResultConsumption` | node | an `#[expect(clippy::let_underscore_must_use)]` on anything but a `let _ = …;` statement |
| `DiModuleLifecycle` | node | a DI module method named `start`, `reload` or `stop`, and `.await`, `block_on` or `.join(` in an `on_disable` |
| `RootModuleConstruction` | node | a constructor call in the root module's `new` whose type is not a `*Module` |
| `SystemClockOutsideDi` | node | a `SystemClock` constructed outside `di` and the test tree; a `SystemClock` declared outside a `system` module; a trait `Clock` that does not declare exactly `fn now(&self) -> SystemTime` |
| `TimePrimitive` | node | Instant in project fields/returns and integer temporal constants; primitive value intent and external boundaries remain review |
| `ModelDerives` | node | a model struct without `Debug`, `Clone` and `PartialEq`, and a `*Id` without `Debug, Clone, PartialEq, Eq, Hash` |
| `ValueObjectShape` | node | for a model type whose `new` calls `assert!`: a non-private field; a statement in `new` other than `assert!` calls and the construction; a field without an accessor of the same name; a `parse_<type>` function outside `util` |
| `ContractShape` | node | contract supertraits/generics, missing async_trait, non-BoxStream stream contracts, callback/task handles and external error types |
| `ErrorFamilyShape` | node | an error enum without `Debug` and `thiserror::Error`; a variant without `#[error]`; a `#[source]` field not typed `Box<dyn Error + Send + Sync>` |
| `ErrorLogging` | node | a `tracing` or `log` macro inside an `Err(<name>)` arm or `if let Err(<name>)` that uses `<name>` only in the format string |
| `CatchUnwind` | node | `catch_unwind` whose closure calls a crate-local item |
| `DocCommentScope` | node | application docs, inherited trait-method docs, missing published error/safety contracts and proven panic docs; unknown receiver panic behavior remains review |
| `GlobalState` | node | a `static` typed `OnceLock`, `LazyLock`, `Mutex` or `RwLock`; `lazy_static!`; installing a logging subscriber (`set_global_default`, `.init()` on a subscriber builder) outside `fn main` |
| `DiModuleShape` | node | a DI module that is not a struct or constructs services outside `new`; a generic parameter bounded by a crate trait on a DI module, use case or entry-point type; a deferred field typed other than `OnceLock` |
| `ConstructorDependency` | node | outside `di`, the root module and the trees, a construction (`X::new(…)`, `X { … }` or unit `X`) of a workspace type that implements a workspace trait, inside `new`, `with_*`, `from_*`, `Default::default` or another type's methods |
| `MainShape` | node | an entry-point `fn main` lacking a `Builder::new_*` runtime, one `block_on` whose block constructs the root module and calls `on_enable`, or `on_disable` after `block_on` |
| `BlockingCallInAsync` | node | `std::thread::sleep`, `std::fs::*` or `std::sync::mpsc::*` inside an `async fn` or `async` block |
| `BroadcastLag` | node | `while let Ok(…) = <x>.recv().await` |
| `SelectCancelSafety` | node | a branch future of `tokio::select!` inside a loop that calls `read_exact`, `read_to_end`, `write_all` or `lock` |
| `JoinSetBehindLock` | node | a type `Mutex<JoinSet<…>>` or `RwLock<JoinSet<…>>` |
| `BlockingLockInSync` | node | `blocking_lock()` in `fn drop` or in `on_disable` |
| `CriticalSectionWrite` | node | a guard scope (`let mut g = <x>.lock()` or `.write()`) with more than one write through `g`, or a statement after the write |
| `LockChoice` | node | a `Mutex` or `RwLock` (any crate) of `bool` or an integer |
| `TestTimePaused` | node | a test calling `tokio::time::sleep`, `interval` or `timeout` without `start_paused = true`; `start_paused` with `flavor = "multi_thread"`; `start_paused` in a crate without tokio's `test-util` in `[dev-dependencies]` |
| `ModelRuleTest` | node | `#[tokio::test]` in a test file that mirrors a `util` file |
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

## Development

```text
rules-rust/
  Cargo.toml, clippy.toml, rust-toolchain.toml, .cargo/config.toml
  modules/rules/   the published crate `klibs-rules`: src/, test/ (self-check, case workspaces, both gate halves)
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
  in one commit, which abandons the drifted version. 0.1.0 is published by hand from its tag with the owner's token
  read from the environment, after which the Trusted Publisher is registered.
