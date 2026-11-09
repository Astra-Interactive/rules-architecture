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
    let report = match check_module.check_workspace_use_case.invoke(workspace_dir) {
        Ok(report) => report,
        Err(error) => {
            println!("{error}");
            return ExitCode::from(2);
        }
    };
    println!("{}", report.to_violation_lines());
    if let Err(error) = check_module.report_writer.write(&report) {
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
`<target>/klibs-rules/violations.txt` and `<target>/klibs-rules/policy.txt`; it exits 0 when clean, 1 with findings, 2 on
a configuration, metadata or write error. No test writes a report file.

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

A crate's role is its declared `role`, else the first matching row. 0.1.0 detects rows 1–3; every other crate needs
no role yet, since no 0.1.0 rule reads one.

| # | Role | Detected when not declared |
|---|---|---|
| 1 | `tooling` | the package is named `xtask` |
| 2 | `proc-macro` | its lib target is a `proc-macro` |
| 3 | `entry-point` | under `instances/`, or no Rust library target (`lib`, `rlib`, `dylib`, `proc-macro`): a cdylib-only or staticlib-only crate is an entry point, since no crate can use it (E0433) |
| 4–15 | `core-ui`, `core-platform`, `core-api`, `core`, `contract`, `storage-contract`, `api`, `impl`, `impl-ui`, `platform`, `technology`, `context` | phase 3 |

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
| `UnitReadme` | crate | a member without `README.md`, a README whose sentence count is not one (counted as Kotlin counts), `readme` set to another path or `false` |
| `RootSources` | workspace, subject `.` | a `[package]` table in the root manifest; a `src/` at the root |
| `ForbiddenPackageName` | module | a module named in `arch-package.md`'s list, `prelude`, `composable`, `theme`, `viewmodel`; `std`, `core`, `alloc`; a raw identifier; `test` or `fixture` other than the two blocks; `bin` directly under `src/` |
| `AnonymousTuple` | node | a tuple type of two or more elements anywhere but the signature of a method of an `impl Trait for` block; a tuple struct or variant of two or more fields |
| `WildcardImport` | node | every `use` tree with a glob |
| `ForbiddenComment` | none | region markers, separator lines and section headers in comments and doc comments (fences and thematic breaks in doc comments aside); generated files skipped |
| `Unanalysable` | node, crate | a root file or file module that does not parse, a missing module file, `include!`, `cfg_attr(…, path = …)`, an item macro call in the root file |
| `NestedUnit` | crate | a member directory nested inside another member |
| `ContainerSources` | workspace | a container holding a manifest, source directory or Rust file |
| `UnitPath` | crate | tooling outside `xtask`, entry points outside `instances/<name>`, other members outside `modules/`, a `shared` path segment, an unregistered `xtask` |
| `ProjectDirMatchesPath` | workspace | a wildcard in a workspace member path |
| `RootBuildSubprojects` | crate | literal shared package settings or dependencies instead of workspace inheritance; profiles, patches or replacements in a member |
| `CrateTargets` | crate | custom library or binary roots, mixed library/binary targets, multiple binaries or `src/bin/` |
| `UnitDependencyCycle` | none | a strongly connected component of normal, build and development edges containing a development edge |
| `EntryPointDependency` | none | any dependency on an entry point or a member without a Rust library target |
| `RootFileShape` | node, file | a root item outside the ordered module tree and platform items, modules in file modules, an invalid fixture block or an undeclared Rust file; proc-macro exports are allowed |
| `ModuleVisibility` | module | visible file modules or inline visibility inconsistent with reexports and test boundaries |
| `TestMirror` | node, module, crate | tests outside the mirrored test tree, a test without a source owner or behaviour, a misplaced test block or path attribute; proc-macro expansion tests may use `tests/it/` |
| `PlatformPackage` | module | platform names used as modules in a crate with target predicates or target dependencies |
| `UtilFileName` | module | a file named `utils.rs`, `helpers.rs` or `extensions.rs`; generated files skipped |
| `FakePlacement` | crate | a separate fake crate instead of the owning crate's `fixture/` feature |
| `ClosureParameterName` | node | a one-letter closure parameter, including destructured bindings |
| `ExplicitItName` | node | an `it` binding, parameter, field or loop label |
| `FunctionVerb` | node | a function starting with `get_`, `map`, `make`, `do`, `handle` or `process`; trait methods, tests and the four collection lookup names keep their names |
| `ImplementationName` | node | `Impl` or `Interface` suffixes and interface `I` prefixes |
| `KindNameInPackage` | node | utility type suffixes or `Model`/`Entity` repeated in domain model modules |
| `TestFunctionName` | node | a test name outside `given_<state>_when_<action>_then_<outcome>` |
| `DeclarationOrder` | node | constants, type families, constructors or trait associated items out of their required order |
| `CalleeBeforeCaller` | node | a caller above its callee in one file or impl; constructors and mutual recursion exempt |
| `OneTypeFamilyPerFile` | node | a mismatched file name, multiple behavioural types, unrelated secondary types or free functions beside a type; `<Type>Factory` is exempt |
| `ConstantOutsideImpl` | node | a type's constants outside or before the end of its inherent impl |
| `StructDefaultValue` | node | default derives or impls on project structs/enums, except deserialized types and argument-less constructors in the type's own file |
| `ImportOrdering` | node | use declarations after other items or separated into multiple blocks |
| `ExpectScope` | node | a lint expectation outside the prescribed lint list or smallest item, including inner and module-level expectations |

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
- CI: `rust-pr.yml` (floor 1.93.0 through `RUSTUP_TOOLCHAIN`, and the pinned toolchain with the gate sync and
  `cargo llvm-cov --fail-under-lines 100`), `rust-release.yml`, `rust-msrv.yml` (weekly fresh resolution of
  `cases/consumer/fixed` on 1.93.0), `rust-doc-drift.yml` (weekly gate sync). None is a required check.
- Release: bump `version` in `modules/rules/Cargo.toml`; on merge `rust-release.yml` finds the commit that set the
  version, runs the gate sync on it, tags it `rules-rust-<version>`, publishes from the tag through crates.io Trusted
  Publishing (`rust-lang/crates-io-auth-action@v1`) and creates the GitHub release; every step resumes after a failure.
  When the documents drifted after the bump, the release stops before tagging: re-sync the gate and bump the version
  in one commit, which abandons the drifted version. 0.1.0 is published by hand from its tag with the owner's token
  read from the environment, after which the Trusted Publisher is registered.
