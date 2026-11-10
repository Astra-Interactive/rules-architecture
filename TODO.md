# TODO: Rust rules the documents state and the linter lacks

What `.claude/docs` says and the Rust linter (`rules-rust`, crate `klibs-rules`) does not check, or checks
wrongly, at `master` `7c2d81e`. Sections are named after the document; every item quotes the document's words.

## How to read an item

- **Kind.** `missed`: a violation the linter does not report. `false positive`: conforming code it reports.
  `no rule`: nothing in the linter looks at the requirement.
- **Status.**
  - `test written`: a test for the case sits uncommitted in the working tree; it has never been run and the
    rule is not fixed.
  - `undecided`: waits for a decision (the brainstorm of rounds 9–10 never ran); no test.
  - `closed by decision`: a recorded decision (`F..` in `decisions.md`) or the audit's root left it out of the
    linter; it is still not in the code.
  - `not mechanical`: the audit found no rule and judged that a checker cannot tell the case without a
    classifier.
  - `candidate (round 11)`, `candidate (round 12)`: found by the auditors of those rounds after this list was
    first written; no decision and no test. The confidence is the auditor's own. Round 12 opened the test files
    behind the round 11 candidates: no test pins any of those forms either way, except 11A05, which a test pins
    as reported.
- **Doc** lines are those of `.claude/docs` on 2026-11-10 and were read for this file. **Rule** paths are under
  `rules-rust/modules/rules/src/` and **Test** paths under `rules-rust/modules/rules/test/`; both are copied from
  the audit and were not reopened.
- Every case of rounds 9–12 comes from reading the linter's source. None is confirmed by a run.

Sources, under `.claude/reports/rust-linter-research/`: the ledger `uncovered-rust-doc-cases.md` (statuses),
`reaudit-2026-11-10/seat-{t,s,a}.md`, `r10-seat-{t,s,a}.md`, `r11-seat-{t,s,a}.md` and `r12-seat-{t,s,a}.md`
(the write-up of each case under `#### <ID>`), `claude-tdd-tracker.md` (case → test), `decisions.md` (F1–F31).

Totals: 111 items. 47 cases of rounds 9–10 (27 `test written`, 20 `undecided`; 9A01 is half and half), 7
`closed by decision`, 9 `not mechanical`, 22 `candidate (round 11)`, 26 `candidate (round 12)`.

## arch-context.md

- [ ] **10T11** — false positive (document conflict) · undecided
  - Doc (`arch-context.md:8–10`): "a use of an `api` unit or of the storage contracts unit (its contracts,
    models, model rules in `util`, use cases, and its `stub` and `memory` implementations)"
  - Doc (`arch-package.md:55–56`): "a feature others depend on is an `api` unit (contracts, the models in their
    signatures, the `util` rules of those models, stubs)"
  - Gap: `KindShape` rejects a `memory` kind in an app `api` crate; one document names it as `api` content and
    the other does not.
  - Rule: `KindShape`, `module/rule/kind_shape.rs:127–129`
- [ ] **10T04** — missed · undecided
  - Doc (`arch-context.md:91–93`): "permits a nested `fun interface Factory` in an `api` contract (in Rust, a
    `<Type>Factory` trait in the contract's file, `arch-domain.rs.md`)"
  - Doc (`arch-package.md:200`): "A domain contract sits in the `api` package of its concept."
  - Gap: a `<Type>Factory` trait is accepted in any kind, beside a struct or enum that is not a contract.
  - Rule: `ContractPlacement`, `module/rule/contract_placement.rs:225–235`
- [ ] **T10** — missed · closed by decision (excluded as manual review: the role cannot be told by name)
  - Doc (`arch-context.md:60`): "`core` holds no use case and no repository, and depends on no feature unit."
  - Gap: a concrete `InvoiceRepository` struct doing file I/O in `core` is accepted; only traits with the
    `Repository` suffix are checked.
  - Rule: `CoreShape`, `module/rule/core_shape.rs:39–57`

## arch-domain.md

- [ ] **9S05** — missed · test written
  - Doc (`arch-domain.md:101–102`): "The word `Service` appears in no type name, except for a component the
    platform runs on its own lifecycle, which lives in `service/`."
  - Gap: `Service` inside a type name is checked only in `model`; `ServiceClient` in `api` is accepted.
  - Rule: `KindNameInPackage`, `naming/rule/name_visitor.rs:123–125`
  - Test: `naming/rule/kind_name_in_package_test.rs` ·
    `given_service_word_inside_non_model_type_name_when_checked_then_reports_only_whole_word`
- [ ] **10S03** — missed · undecided
  - Doc (`arch-domain.md:45`): "A domain type carries no kind suffix: `Jail`, not `JailModel` or `JailEntity`."
  - Doc (`clean-code.rs.md:61`): "No `Helper`, `Util` or `Manager` types"
  - Gap: a type alias is never checked for `Model`, `Entity`, `Helper`, `Manager`, `Impl` or `Service`.
  - Rule: `KindNameInPackage`, `naming/rule/name_visitor.rs:228–252`
- [ ] **T18** — missed · closed by decision (F26: not reported, as in the Kotlin rule)
  - Doc (`arch-domain.md:19–21`): "It is never a member of the type …, and never a top-level function that
    takes the type as a parameter."
  - Gap: a free `util` function taking one model plus a non-model parameter
    (`fn is_expired(invoice: Invoice, now: SystemTime) -> bool`) is accepted; only one-parameter functions are
    checked.
  - Rule: `UtilFunctionAsExtension`, `module/rule/util_function_as_extension.rs:18–23`
- [ ] **11S03** — missed · candidate (round 11), confidence low to medium
  - Doc (`arch-domain.md:101–102`): "The word `Service` appears in no type name, except for a component the
    platform runs on its own lifecycle, which lives in `service/`."
  - Gap: the exemption is tested against the words of the crate name too, so every type of a crate named
    `…-service` may carry `Service` (`PaymentService` in `usecase`).
  - Rule: `KindNameInPackage`, `naming/rule/name_visitor.rs:93–134`
- [ ] **12S06** — missed · candidate (round 12), confidence low to medium
  - Doc (`arch-domain.md:37–38`): "It carries a typed identifier (`PlayerId`, `JailId`), never a bare string or
    number."
  - Gap: a field named `id` or `*_id` of type `Cow<str>`, `Rc<str>`, `Arc<String>` or a `NonZero*` integer is
    not treated as a bare identifier.
  - Rule: `BareIdentifier`, `declaration/rule/bare_identifier.rs:38–94`

## arch-domain.rs.md

- [ ] **10S07** — missed · undecided
  - Doc (`arch-domain.rs.md:172–173`): "A publisher contract in `api` hands it out as a stream …:
    `fn events(&self) -> BoxStream<'static, InvoiceEvent>`"
  - Gap: the `'static` stream form is enforced only on a contract method named `events`; `BoxStream<'_, _>`
    from a method with another name is accepted.
  - Rule: `ContractShape`, `declaration/util/contract_method_shape.rs:228–232`
- [ ] **12S01** — false positive (the document's own example) · candidate (round 12), confidence medium
  - Doc (`arch-domain.rs.md:9–11`): "They derive `Debug`, `Clone` and `PartialEq` …, and also `Eq` and `Hash`
    when every field implements them"
  - Doc (`arch-domain.rs.md:25–26`): "#[derive(Debug, Clone, PartialEq, Eq)] pub(crate) struct Invoice"
  - Gap: the Good `Invoice` example derives `Eq` without `Hash` although each of its fields implements both, so
    the linter, which follows the sentence, would report the example. The fix belongs in the document.
  - Rule: `ModelDerives`, `declaration/rule/model_derives.rs:230–287`
- [ ] **12S02** — missed · candidate (round 12), confidence low to medium
  - Doc (`arch-domain.rs.md:64`): "`parse_<type>(raw) -> Result<Type, TypeError>`"
  - Gap: the error type of a `parse_<type>` function is never checked; `Result<Money, String>` and
    `Result<Money, ()>` pass, and a test pins `Result<_, ()>` as valid.
  - Rule: `ValueObjectShape`, `declaration/rule/value_object_shape.rs:29–86`
- [ ] **12S03** — missed · candidate (round 12), confidence low to medium
  - Doc (`arch-domain.rs.md:61–62`): "A value object with an invariant keeps its fields private … Its `new`
    checks the invariant"
  - Gap: a `new` that guards with `if … { panic!() }` instead of `assert!` is not recognised as an invariant
    check, so the type's `pub` field passes; a test pins this form as unreported.
  - Rule: `ValueObjectShape`, `declaration/rule/value_object_shape.rs:65–68,139–176`
- [ ] **12S05** — missed · candidate (round 12), confidence low
  - Doc (`arch-domain.rs.md:19–20`): "No method of a model type takes `&mut self`, and no code assigns a field of
    a model value"
  - Gap: a `self: Pin<&mut Self>` receiver on a model type is not seen as a mutable receiver.
  - Rule: `ModelShouldBeImmutable`, `module/rule/model_should_be_immutable.rs:58–93`
- See also **10S06** under `code-ordering.rs.md` (`arch-domain.rs.md:100–101`: "A struct that only its variants
  carry sits in its file.").

## arch-package.md

- [ ] **9T06** — missed · test written
  - Doc (`arch-package.md:214–215`): "A framework registration interface stays in the package of its mechanism
    (`route`, `command`, or the project's equivalent), including in mixed `core`"
  - Gap: a registration-shaped trait is accepted in `model` or `util` of a `core`-role crate.
  - Rule: `ContractPlacement`, `module/rule/contract_placement.rs:247–257`
  - Test: `module/rule/contract_placement_test.rs` ·
    `given_registration_trait_in_core_model_when_checked_then_only_it_reports_and_the_mechanism_package_passes`
- [ ] **10T02** — false positive · test written
  - Doc (`arch-package.md:211`): "`database`, `network`, `local` and `remote` are not technology names."
  - Gap: `LocalizedMessageRepository` and `Locale…` are reported as the forbidden `Local` prefix; the match has
    no word boundary.
  - Rule: `ImplementationName`, `naming/rule/implementation_name.rs:50–54`
  - Test: `naming/rule/implementation_name_test.rs` ·
    `given_locale_prefixed_implementations_when_checked_then_reports_only_the_local_prefix`
- [ ] **10T03** — missed · test written
  - Doc (`arch-package.md:255–256`): "Every error family is its own file in `model`, a sealed error with all its
    subclasses or a single error class"
  - Gap: an error family declared with a hand-written `impl std::error::Error` is not recognised as one, so its
    placement is not checked.
  - Rule: `ErrorPlacement`, `module/rule/error_placement.rs:13–25`
  - Test: `module/rule/error_placement_test.rs` ·
    `given_hand_written_error_family_when_declared_outside_model_then_reports_it`
- [ ] **10T09** — missed · test written
  - Doc (`arch-package.md:255–256`): "Every error family is its own file in `model`, … or a single error class"
  - Gap: a nested error struct in the file of the type that uses it is accepted (the enum form is already
    reported).
  - Rule: `OneTypeFamilyPerFile`, `family/rule/one_type_family_per_file.rs:179–184`
  - Test: `family/rule/one_type_family_per_file_test.rs` ·
    `given_error_struct_declared_beside_the_struct_using_it_when_checked_then_reports_only_the_error`
- [ ] **10T06** — false positive · test written
  - Doc (`arch-package.md:178–179`): "Every package name is singular, except an inherently plural concept
    (`news`)"
  - Gap: singular names ending in `-as` (`alias`, `canvas`, `atlas`, `bias`) are reported as plural.
  - Rule: `PluralPackageName`, `module/rule/plural_package_name.rs:13–14`
  - Test: `module/rule/plural_package_name_test.rs` ·
    `given_concept_names_ending_in_as_when_checked_then_reports_only_the_real_plural`
- [ ] **12T02** — missed · candidate (round 12), confidence low (the document leaves plurality to the reviewer)
  - Doc (`arch-package.md:178–179`): "Every package name is singular, except an inherently plural concept
    (`news`); which concept is plural is the reviewer's call."
  - Gap: every name ending in `us` or `is` is taken as singular, so `menus`, `wikis` and `taxis` pass.
  - Rule: `PluralPackageName`, `module/rule/plural_package_name.rs:13–14`
- See also **9T03** under `arch-package.rs.md` (`arch-package.md:75–76`), **10T04** and **10T11** under
  `arch-context.md` (`arch-package.md:200`, `:55–56`), **11A05** and **11S06** under `error-handling.rs.md`
  (`arch-package.md:255–256`).

## arch-package.rs.md

- [ ] **9T02** — missed · test written
  - Doc (`arch-package.rs.md:353–354`): "a release artefact is built without them …, and the fixture tree stays
    out of it."
  - Gap: a crate's own `default = ["fake"]` compiles the fixture tree into a release build and is accepted.
  - Rule: `FakePlacement`, `module/rule/fake_placement.rs:76–100`
  - Test: `module/rule/fake_placement_test.rs` ·
    `given_default_feature_enabling_fake_when_checked_then_reports_the_default_key`
- [ ] **9T03** — missed (document conflict) · undecided
  - Doc (`arch-package.rs.md:171–172`): "A container directory holds crate directories and nothing else: no
    `Cargo.toml`, no `src/`."
  - Doc (`arch-package.md:75–76`): "a split sub-context's directory is a container with no build file and no
    sources"
  - Gap: a `README.md` in a container directory is accepted; the linter follows the more lenient document.
  - Rule: `ContainerSources`, `layout/rule/container_sources.rs:24`
- [ ] **10T13** — missed · undecided (decided together with 9T03)
  - Doc (`arch-package.rs.md:171`): "A container directory holds crate directories and nothing else"
  - Gap: a stray directory that holds no crate is accepted inside a container.
  - Rule: `ContainerSources`, `layout/rule/container_sources.rs:20–26`
- [ ] **9T04** — missed · test written
  - Doc (`arch-package.rs.md:135`): "the functions or the plugin type a host loads from a library are `pub`"
  - Gap: a private `extern "C"` / `no_mangle` function of a `cdylib` is accepted when `host-items` does not
    list it.
  - Rule: `EntryPointShape`, `module/rule/entry_point_shape.rs:56–58`
  - Test: `module/rule/entry_point_shape_test.rs` ·
    `given_cdylib_without_host_items_when_host_function_is_private_then_only_it_reports`
- [ ] **9T05** — missed · test written
  - Doc (`arch-package.rs.md:147–148`): "Each `#[doc(hidden)] pub` item is used by another crate of the
    workspace; the checker reports one that is not."
  - Gap: `#[doc(hidden)] pub` constants, statics and `impl` or trait members are never checked for a workspace
    user.
  - Rule: `DocHiddenUse`, `module/rule/doc_hidden_use.rs:51–58`
  - Test: `module/rule/workspace_visibility_test.rs` ·
    `given_unused_public_hidden_constant_and_method_when_workspace_only_then_reports_each`
- [ ] **10T01** — missed · undecided (the bound of the check)
  - Doc (`arch-package.rs.md:131–133`): "Fields and methods take the tier of their widest user the same way:
    private unless another file uses them … it does not report fields, so the checker does."
  - Gap: a `pub` field or method of a reachable `pub` type that no other file uses is never reported.
  - Rule: `VisibilityTier`, `module/rule/visibility_tier.rs:130–169,486–510`
- [ ] **10T05** — missed · undecided
  - Doc (`arch-package.rs.md:83–84`): "Platform code … is code under a target `cfg` … at the same module path,
    never in a module named after the platform."
  - Gap: a module named after a platform (`linux`) is accepted in a crate that has no target `cfg`.
  - Rule: `PlatformPackage`, `module/rule/platform_package.rs:64–66`
- [ ] **10T08** — missed · test written
  - Doc (`arch-package.rs.md:71–74`): "Its kind block re-exports each of its items once, by name … A type has
    exactly that one path"
  - Gap: a second `pub use` of an already re-exported item in an ancestor block is accepted.
  - Rule: `ReExportShape`, `module/rule/re_export_shape.rs:118–150`
  - Test: `module/rule/re_export_shape_test.rs` ·
    `given_item_re_exported_by_the_concept_block_above_its_kind_block_when_checked_then_the_second_path_is_reported`
- [ ] **10T12** — false positive · test written
  - Doc (`arch-package.rs.md:290–292`): "`<type>_ext.rs` holds the rules of one type: … a trait `<Type>Ext`
    with its one `impl` when another crate declares the type"
  - Gap: an Ext trait for a primitive, a reference or a slice (`impl StrExt for str`) is reported as a wrong
    implementation.
  - Rule: `UtilFileName`, `module/rule/util_file_name.rs:76–176`
  - Test: `module/rule/util_file_name_test.rs` ·
    `given_primitive_and_slice_extension_traits_when_checked_then_reports_only_the_missing_impl`
- [ ] **10T14** — missed · undecided
  - Doc (`arch-package.rs.md:241–243`): "Build automation … is a subcommand of the `xtask` crate …, never a
    shell script."
  - Gap: a shell script with a shebang and no extension is not seen as build automation.
  - Rule: `ShellAutomation`, `layout/rule/shell_automation.rs:8–13`
- [ ] **7T06, package-key half** — missed · closed by decision (F28: about 80 committed fixtures and the clean
  consumer fixture are laid out that way)
  - Doc (`arch-package.rs.md:219–220`): "Shared settings are declared once in the root: `[workspace.package]`,
    `[workspace.dependencies]` and `[workspace.lints]`"
  - Gap: a package key (`edition`, `license`, …) written with the same literal in two members while the root
    declares none is accepted. The dependency half of 7T06 is fixed.
  - Rule: `RootBuildSubprojects`, `layout/rule/root_build_subprojects.rs`
- [ ] **11T03** — missed · candidate (round 11), confidence medium to low (a private double of a foreign trait
  may be meant as allowed)
  - Doc (`arch-package.rs.md:321–323`): "A fake that tests of other crates need lives in the fixture tree of the
    crate that owns the trait … and never copied into each consumer."
  - Gap: a `Fake…` in a consumer crate's test tree that implements another workspace crate's trait is accepted;
    the cross-crate condition is applied to the fixture tree only.
  - Rule: `FakePlacement`, `module/rule/fake_placement.rs:14–74`
- [ ] **12T05** — missed · candidate (round 12), confidence medium
  - Doc (`arch-package.rs.md:122–123`): "the visibility of an item is its tier"
  - Doc (`arch-package.rs.md:131–132`): "private unless another file uses them, and never `pub` on a type that is
    not `pub`"
  - Gap: the table has three tiers (private, `pub(crate)`, `pub`), yet `pub(super)` and `pub(self)` items and
    `pub(super)` or `pub(in …)` fields and methods are accepted; only `pub(in path)` on an item is reported.
  - Rule: `VisibilityTier`, `module/rule/visibility_tier.rs:89–91,130–207`
- [ ] **12T08** — missed · candidate (round 12), confidence medium
  - Doc (`arch-package.rs.md:287`): "A type that is a field type of exactly one type sits in that type's file."
  - Gap: the "used by" lookup counts a type's reference to itself, so a recursive secondary type
    (`struct TreeNode { child: Option<Box<TreeNode>> }`) that the file's primary type never uses passes.
  - Rule: `OneTypeFamilyPerFile`, `family/rule/one_type_family_per_file.rs:186–240`,
    `family/util/type_family.rs:36–54`
- [ ] **12T01** — missed · candidate (round 12), confidence medium to low
  - Doc (`arch-package.rs.md:219–220`): "Shared settings are declared once in the root: `[workspace.package]`,
    `[workspace.dependencies]` and `[workspace.lints]`"
  - Gap: dependencies are compared by key and exact literal, so a renamed dependency
    (`alias = { package = "foo" }` against the root's `foo`) and `foo = "1"` against
    `foo = { version = "1" }` both escape.
  - Rule: `RootBuildSubprojects`, `layout/rule/root_build_subprojects.rs:29–117`
- [ ] **12T04** — missed · candidate (round 12), confidence medium to low (extends 9T05)
  - Doc (`arch-package.rs.md:147–148`): "Each `#[doc(hidden)] pub` item is used by another crate of the
    workspace; the checker reports one that is not."
  - Gap: only structs, enums, traits, type aliases and functions are checked; a `#[doc(hidden)] pub` module,
    re-export, union, macro, field or variant never is.
  - Rule: `DocHiddenUse`, `module/rule/doc_hidden_use.rs:51–58`
- [ ] **12T06** — missed · candidate (round 12), confidence medium to low
  - Doc (`arch-package.rs.md:134–135`): "`fn main` of a binary has none"
  - Gap: `pub(crate) fn main` is accepted; only a bare `pub` is reported.
  - Rule: `EntryPointShape`, `module/rule/entry_point_shape.rs:54–69`
- [ ] **12T07** — missed · candidate (round 12), confidence low to medium
  - Doc (`arch-package.rs.md:357`): "There is no `tests/` directory: every test of a crate is in its `test/`
    tree"
  - Gap: a `[[test]]` target whose path is outside `tests/` is seen by no rule.
  - Rule: `TestMirror`, `module/rule/test_mirror.rs:169–193`; `CrateTargets`, `layout/rule/crate_targets.rs:8–46`
- [ ] **12T03** — false positive (a test pins the code) · candidate (round 12), confidence low
  - Doc (`arch-package.rs.md:195`): "other crates under `modules/core`"
  - Gap: the crate at `modules/core` itself is detected as `core-api`, though the table speaks of crates under
    that directory and gives an unsplit `core` the detection "never"; `role_resolve_test.rs` pins the code.
  - Rule: role resolution, `config/util/role_resolve.rs:75–87`

## async.rs.md

- [ ] **9A03** — missed · test written
  - Doc (`async.rs.md:197–202`): "In a `tokio::select!` that runs in a loop, every branch is cancel safe … :
    `read_exact`, `read_to_end`, `write_all` and `tokio::sync::Mutex::lock` lose data or their place in the
    queue when another branch wins"
  - Gap: `read_exact` in a looping `select!` on a local from `TcpStream::connect(..).await` is accepted; the
    type of an awaited constructor is not inferred.
  - Rule: `SelectCancelSafety`, `runtime/rule/typed_runtime_visitor.rs:770–796`
  - Test: `runtime/rule/select_cancel_safety_test.rs` ·
    `given_read_exact_on_connected_local_in_looping_select_when_checked_then_reports_the_branch`
- [ ] **9A04** — missed · test written
  - Doc (`async.rs.md:213`): "A lock whose guard no `.await` crosses is `parking_lot::Mutex` or
    `parking_lot::RwLock`, in synchronous and asynchronous code alike"
  - Gap: a `tokio::sync::RwLock` whose guard no `.await` crosses is accepted; only `tokio::sync::Mutex` with
    `lock` and `write` is looked at.
  - Rule: `LockChoice`, `runtime/rule/typed_runtime_visitor.rs:838–865,1363`
  - Test: `runtime/rule/lock_choice_test.rs` ·
    `given_lock_choice_tokio_rwlock_guards_without_crossing_await_when_linted_then_reports_both`
- [ ] **seat A row 25** — no rule · not mechanical ("started by a request or user action" needs a classifier)
  - Doc (`async.rs.md:79–80`): "A task started by a request or a user action is guarded atomically against a
    second start. A flag in the state such as `is_busy` is what a reader shows, not a lock"
- [ ] **seat A row 29** — no rule · not mechanical (a project-written type with no fixed form)
  - Doc (`async.rs.md:134–138`): "The scope is a small type written once. … each such type receives an
    `Arc<TaskScope>`."
  - Gap: the shape of `TaskScope` itself is not checked; only its owner is.
- [ ] **seat A row 32** — no rule · not mechanical (choosing the primitive is semantic)
  - Doc (`async.rs.md:210–212`): "State a reader observes is a `watch` channel … A counter or a flag is an
    atomic. A resource that one task can own … is owned by that task and reached through an `mpsc` channel."
- [ ] **11A03** — missed · candidate (round 11), confidence low to medium
  - Doc (`async.rs.md:217–218`): "A synchronous function on a runtime thread (a `Drop`, a synchronous
    `on_disable`) cannot lock it: `blocking_lock` panics there."
  - Gap: only `Mutex::blocking_lock` inside `drop` or `on_disable` is reported. `RwLock::blocking_read` and
    `blocking_write` are not looked at, nor are `on_enable` and `on_reload`. Document conflict:
    `di-manual.rs.md:100–106` runs `on_disable` after `block_on`, off the runtime.
  - Rule: `BlockingLockInSync`, `runtime/rule/typed_runtime_visitor.rs:1268–1310,1455–1461`
- [ ] **11A11** — missed · candidate (round 11), confidence low to medium
  - Doc (`async.rs.md:11–13`): "It never wraps a read that returns at once, `wait_for(|_| true)` or an `async`
    block with no `.await` in it"
  - Gap: `tokio::time::timeout_at` is not named, and `timeout(d, async { receiver.wait_for(|_| true).await })`
    passes because the block holds an `.await`.
  - Rule: `WatchTimeout`, `runtime/rule/typed_runtime_visitor.rs:968–1010,1508–1511`
- [ ] **11A09** — missed · candidate (round 11), confidence low (the document names `tokio::spawn` only)
  - Doc (`async.rs.md:122–123`): "`tokio::spawn` whose `JoinHandle` is dropped (a bare statement, `let _ =`,
    `drop(…)`) detaches the task: nothing stops it, and its panic reaches no one."
  - Gap: `tokio::task::spawn_local` and `tokio::task::Builder::spawn` are not in the set of spawn paths, while
    `spawn_blocking`, which the document does not name either, is.
  - Rule: `DetachedTask`, `runtime/rule/task_visitor.rs:172–178,277–301`
- [ ] **11A07** — missed · candidate (round 11), confidence low (the reason is explanatory text)
  - Doc (`async.rs.md:224–225`): "`clippy.toml` lists `std::sync::Mutex`, `std::sync::RwLock` and
    `std::sync::PoisonError` under `disallowed-types`, with this rule as the reason."
  - Gap: the required-configuration check compares the entry's `path` and `allow-invalid` only; a missing
    `reason` is accepted.
  - Rule: `LintConfig`, `config/rule/lint_config.rs:327–370`
- [ ] **12A01** — missed · candidate (round 12), confidence medium
  - Doc (`async.rs.md:44–45`): "The closure of an atomic's `fetch_update` is a compare-and-set loop that may run
    several times (std), so it is pure and writes nothing it captures"
  - Gap: a `fetch_update` closure that writes a capture through `Option::take`, `Cell::set`,
    `String::push_str` or `mem::replace` is not seen as impure; the list of mutating methods is short.
  - Rule: `ImpureUpdateClosure`, `runtime/rule/typed_runtime_visitor.rs:730–768`,
    `runtime/rule/closure_effects.rs:62–128`
- [ ] **12A02** — missed · candidate (round 12), confidence medium
  - Doc (`async.rs.md:232–234`): "A critical section computes first and writes once. … Then it stores the value
    in one statement, the last one in the guard's scope: `*cache = next;`, or one `insert` or `remove` on the
    guarded collection."
  - Gap: writes are counted by a list of method names that lacks `push_back`, `pop_front`, `entry`,
    `push_str`, `sort_by` and `mem::replace`, so two such writes under one guard pass.
  - Rule: `CriticalSectionWrite`, `runtime/rule/guard_writes.rs:100–125`
- [ ] **12A07** — missed · candidate (round 12), confidence low to medium
  - Doc (`async.rs.md:122–123`): "`tokio::spawn` whose `JoinHandle` is dropped (a bare statement, `let _ =`,
    `drop(…)`) detaches the task"
  - Gap: `Handle::current().spawn(..)` and `runtime.handle().spawn(..)` are not recognised as a spawn; the
    receiver must be a typed local or `self.field`.
  - Rule: `DetachedTask`, `runtime/rule/task_visitor.rs:218–262`
- [ ] **12A11** — missed · candidate (round 12), confidence low (tokio's behaviour is the auditor's recollection)
  - Doc (`async.rs.md:201–202`): "`read_exact`, `read_to_end`, `write_all` and `tokio::sync::Mutex::lock` lose
    data or their place in the queue when another branch wins"
  - Gap: `RwLock::read`, `RwLock::write` and `Semaphore::acquire` in a looping `select!` are not named.
  - Rule: `SelectCancelSafety`, `runtime/rule/typed_runtime_visitor.rs:770–796`

## clean-code.rs.md

- [ ] **T16** — missed · closed by decision (excluded as manual review: one field may itself be a collection)
  - Doc (`clean-code.rs.md:15–20`): "one that returns a value is … never starts with `get_`. These keep their
    names: … `get`, `get_mut`, `get_unchecked` and `get_unchecked_mut` as the lookup of a collection type"
  - Gap: `Account::get(&self)` that returns a field of an ordinary struct is accepted; the rule leaves the
    lookup role to review.
  - Rule: `FunctionVerb`, `naming/rule/name_visitor.rs:157–165`
- See also **10S03** under `arch-domain.md` (`clean-code.rs.md:61`).

## closure-naming.rs.md

- [ ] **10S04 (F17)** — missed · closed by decision (F17 fixed `matches!` only; the audit's root dropped the
  rest as the same recorded mechanism)
  - Doc (`closure-naming.rs.md:6–7`): "Never name anything `it`: not a closure parameter, not a function
    parameter, not a `let` or `match` binding, not a field, not a loop label"
  - Gap: names inside a macro whose tokens are not a comma list of expressions (`tokio::select!`) are never
    visited.
  - Rule: `ExplicitItName`, `macro_arguments.rs:7–13`

## code-ordering.rs.md

- [ ] **10S06** — false positive (to confirm) · undecided (the rule's own test pins an inherent `impl` as a
  violation)
  - Doc (`code-ordering.rs.md:10–11`): "Then the file's type: its definition, its inherent `impl` block, then
    its trait `impl` blocks, then the data types only it uses, each with its own `impl` blocks."
  - Doc (`arch-domain.rs.md:100–101`): "A struct that only its variants carry sits in its file."
  - Gap: a trait `impl` or an inherent `impl` with functions on a payload type in its owner's file is reported
    as a second type with behaviour.
  - Rule: `OneTypeFamilyPerFile`, `family/rule/one_type_family_per_file.rs:113–147,225–229`

## di-manual-constructors.rs.md

- [ ] **10A06** — missed · undecided
  - Doc (`di-manual-constructors.rs.md:10–11`): "Dependencies must be created only at the composition root / DI
    module, then passed down from parent to child through `new`."
  - Gap: a concrete dependency built in `main` or in a free function is accepted.
  - Rule: `ConstructorDependency`, `module/rule/constructor_dependency.rs:11–63`
- [ ] **12A09** — missed · candidate (round 12), confidence low
  - Doc (`di-manual-constructors.rs.md:3–7`): "Never instantiate concrete dependencies inside a type's `new` … A
    dependency is a value with behaviour the type calls and a test might replace (a repository, a client, a
    clock)."
  - Gap: only implementations declared in the workspace are seen; `reqwest::Client::new()` inside `new` passes,
    while `RootModuleConstruction` does report calls into dependency crates.
  - Rule: `ConstructorDependency`, `module/rule/constructor_dependency.rs:71–113`

## di-manual.rs.md

- [ ] **9A05** — missed · undecided (needs a rule other than a crate list)
  - Doc (`di-manual.rs.md:143–145`): "no `static` holding a service or shared state (a `static` of `OnceLock`,
    `LazyLock`, `Mutex` or `RwLock`, `lazy_static!`). … A `static` holds only constant data."
  - Gap: a `static` of third-party shared state (`arc_swap::ArcSwapOption`) is accepted.
  - Rule: `GlobalState`, `module/rule/global_state_visitor.rs:66–98`
- [ ] **10A02** — missed · test written
  - Doc (`di-manual.rs.md:11–12`): "A contract is never a type parameter of a DI module, a use case or an entry
    point"
  - Gap: a contract bound written on the `impl` (`impl<R: Contract>`) of a DI module or entry-point type is not
    seen.
  - Rule: `DiModuleShape`, `module/rule/di_module_shape.rs:365–405`
  - Test: `module/rule/di_module_shape_test.rs` ·
    `given_contract_bound_on_di_module_impl_when_checked_then_the_struct_is_reported`
- [ ] **10A03** — false positive · test written
  - Doc (`di-manual.rs.md:54–56`): "A DI module that starts or stops something implements the project's
    `Lifecycle` trait …, and its methods call the same method of only the DI modules it owns, in order."
  - Gap: a parent is reported for not forwarding to a child module that has no `Lifecycle`.
  - Rule: `DiModuleLifecycle`, `module/rule/di_module_lifecycle.rs:297–298,451–454`
  - Test: `module/rule/di_module_lifecycle_test.rs` ·
    `given_parent_owning_a_child_without_lifecycle_when_checked_then_only_the_unforwarded_lifecycle_child_reports`
- [ ] **seat A row 1** — no rule · not mechanical (would need an invented list of DI crates)
  - Doc (`di-manual.rs.md:3`): "Manual constructor injection: no DI framework crate and no service locator."
  - Gap: no rule names a DI framework crate. The service-locator half is `GlobalState`.
- [ ] **seat A row 13** — no rule · not mechanical (needs a classifier of what a task does)
  - Doc (`di-manual.rs.md:141–142`): "A task that seeds or migrates a resource (`warmup`, `migration`) is built
    by the DI module of that resource and started by its `on_enable`."
- [ ] **11A06** — missed · candidate (round 11), confidence low (is a constructor's type parameter "a type
  parameter of a DI module"?)
  - Doc (`di-manual.rs.md:11–12`): "A contract is never a type parameter of a DI module, a use case or an entry
    point"
  - Gap: only the generics of the struct or enum are read; `fn new<R: InvoiceRepository>(repository: R)` and an
    `impl Clock` parameter of a DI module's constructor are not seen. 10A02 is the `impl<R: Contract>` bound.
  - Rule: `DiModuleShape`, `module/rule/di_module_shape.rs:365–405,441–477`
- [ ] **11A08** — missed · candidate (round 11), confidence low
  - Doc (`di-manual.rs.md:145–146`): "The one global is the logging subscriber, which `main` installs once"
  - Gap: only the installers of `tracing`, `tracing_subscriber` and `log` are known; `env_logger::init()`,
    `fern::Dispatch::apply()` and `tracing_log::LogTracer::init()` outside `main` are accepted.
  - Rule: `GlobalState`, `module/rule/global_state_visitor.rs:56–64,165–186`
- [ ] **12A08** — false positive · candidate (round 12), confidence low to medium
  - Doc (`di-manual.rs.md:101–102`): "calls `on_enable` and waits for the shutdown signal"
  - Gap: a `main` that waits through a helper (`shutdown_signal().await`) or a `CancellationToken` is rejected;
    only a direct `tokio::signal` await or a signal's `recv()` counts as the wait.
  - Rule: `MainShape`, `module/rule/main_shape_visitor.rs:62–83,159–176`
- [ ] **12A10** — missed · candidate (round 12), confidence low
  - Doc (`di-manual.rs.md:143–145`): "no `static` holding a service or shared state … A `static` holds only
    constant data."
  - Gap: the statics declared inside `thread_local!` are not read.
  - Rule: `GlobalState`, `module/rule/global_state_visitor.rs:150–167`

## error-handling.rs.md

- [ ] **9A01** — missed · `_ = call();` half: test written; `let _name = call();` half: undecided
  - Doc (`error-handling.rs.md:155–156`): "never discard a `Result`: no call whose `Result` is left unused, and
    no `let _ =` on it, with one exception."
  - Gap: a `Result` discarded by `_ = call();` or `let _name = call();` has no rule. That clippy's
    `let_underscore_must_use` covers only `let _ =` is the auditor's recollection, not checked.
  - Rule: `ResultConsumption`
  - Test: `runtime/rule/result_consumption_test.rs` ·
    `given_result_consumption_underscore_assignment_of_workspace_result_when_linted_then_reports_the_statement`
- [ ] **9A02** — false positive · test written
  - Doc (`error-handling.rs.md:156–158`): "The send of a channel whose failures the design accepts may be
    discarded: `oneshot::Sender::send`, `mpsc::Sender::try_send`, `mpsc::UnboundedSender::send` and
    `broadcast::Sender::send`."
  - Gap: the allowed discard is reported when the sender is reached through a nested field
    (`self.channels.events.try_send(..)`).
  - Rule: `ResultConsumption`, `runtime/rule/typed_runtime_visitor.rs:389–392`
  - Test: `runtime/rule/result_consumption_test.rs` ·
    `given_result_consumption_sender_through_nested_field_when_linted_then_reports_only_the_real_discard`
- [ ] **10A07** — missed · test written
  - Doc (`error-handling.rs.md:85–87`): "translate its error into a variant of the family right there, so inner
    layers never see a foreign error type."
  - Gap: a contract method returning a one-argument external alias (`std::io::Result<T>`, `anyhow::Result<T>`)
    escapes the foreign-error check.
  - Rule: `ContractShape`, `declaration/util/contract_method_shape.rs:104–153`
  - Test: `declaration/rule/contract_shape_test.rs` ·
    `given_contract_method_returning_one_argument_external_result_when_checked_then_the_trait_reports`
- [ ] **seat A row 39** — no rule · not mechanical (needs a classifier; clippy `panic` is not among the
  required lints)
  - Doc (`error-handling.rs.md:12`): "Never panic for an expected business or validation failure: a panic is
    reserved for bugs."
- [ ] **11A01** — missed · candidate (round 11), confidence medium
  - Doc (`error-handling.rs.md:101–102`): "Your own logic returns `Err` and is never wrapped in
    `std::panic::catch_unwind`"
  - Gap: only a closure argument is inspected; `catch_unwind(run_batch)` and
    `catch_unwind(AssertUnwindSafe(run_batch))` with a workspace function passed by name are accepted.
  - Rule: `CatchUnwind`, `runtime/rule/task_visitor.rs:312–342`, `runtime/rule/closure_effects.rs:24–42`
- [ ] **11A02** — missed · candidate (round 11), confidence medium (does "discard" include `.ok();`?)
  - Doc (`error-handling.rs.md:155–156`): "never discard a `Result`: no call whose `Result` is left unused, and
    no `let _ =` on it, with one exception."
  - Gap: `save_order(order).ok();` and `let _ = save_order(order).ok();` drop the error and have no rule. That
    no required lint fires on the resulting `Option` is the auditor's recollection, not checked.
  - Rule: `ResultConsumption`, `runtime/rule/result_consumption.rs:26–54,90–107`
- [ ] **11A05** — false positive (document conflict) · candidate (round 11), confidence low to medium
  - Doc (`arch-package.md:255–256`): "Every error family is its own file in `model`, a sealed error with all its
    subclasses or a single error class"
  - Doc (`error-handling.rs.md:26–27`): "Model domain errors as one `enum` per family deriving `Debug` and
    `thiserror::Error`"
  - Gap: every struct that derives or implements `Error` is reported with "an error family is an enum, not a
    struct", while 10T03 and 10T09 ask the placement rules to accept and place such a struct.
  - Rule: `ErrorFamilyShape`, `declaration/rule/error_family_shape.rs:124–142`
- [ ] **11S06** — false positive · candidate (round 11), confidence low to medium
  - Doc (`arch-package.md:255–256`): "or a single error class"
  - Doc (`error-handling.rs.md:28–29`): "A cause from code you do not own is the `#[source]` field
    `source: Box<dyn Error + Send + Sync>`"
  - Gap: every struct in `model` must derive `Clone` and `PartialEq` (and `Eq`, `Hash`); an error struct with a
    boxed source cannot derive them and is reported. Enums are not checked, so the error enum passes.
  - Rule: `ModelDerives`, `declaration/rule/model_derives.rs:230–287`

## imports.rs.md

- [ ] **9T01** — missed · test written
  - Doc (`imports.rs.md:10–12`): "A comment at the end of a `use` line is not allowed either: the one on the
    last line of the block stays in place when rustfmt reorders and ends up on a different import"
  - Gap: a trailing comment on the last `use` line of the block is not reported.
  - Rule: `ImportOrdering`, `import/rule/import_order_visitor.rs:34–53`
  - Test: `import/rule/import_ordering_test.rs` ·
    `given_trailing_comment_on_last_import_when_checked_then_reports_that_import`
- [ ] **11T02** — no rule · candidate (round 11), confidence low to medium (is `cargo fmt` a CI step of the
  consumer or a check of the linter?)
  - Doc (`imports.rs.md:7–8`): "in the order `cargo fmt` gives: rustfmt sorts a block of consecutive `use` lines"
  - Gap: nothing compares the order of the `use` paths; only gaps, comments and position are checked, and no
    required configuration makes a consumer run `cargo fmt --check`.
  - Rule: `ImportOrdering`, `import/rule/import_order_visitor.rs:34–53`
- [ ] **11T04** — missed · candidate (round 11), confidence low (the document words the rule for a file)
  - Doc (`imports.rs.md:7`): "The imports of a file are one block at its top, with no blank line and no comment
    inside it"
  - Gap: a blank line or a comment between the `use` lines of an inline module block (a kind block of the root
    file) is not reported; the gap check runs for file scopes only.
  - Rule: `ImportOrdering`, `import/rule/import_order_visitor.rs:34–53,69–76`
- See also **7T05** under `notes.rs.md` (`imports.rs.md:3`: "Never use wildcard imports").

## no-callbacks.rs.md

- [ ] **10A05** — missed · test written
  - Doc (`no-callbacks.rs.md:274–275`): "`std::thread::spawn` and `std::thread::Builder::spawn` start only
    blocking work that runs until shutdown, in a technology module"
  - Gap: `std::thread::Builder::new().name(..).spawn(..)` outside a technology module is accepted; the
    `Builder` type is lost after `.name(..)`.
  - Rule: `ThreadPrimitive`, `runtime/rule/typed_runtime_visitor.rs:225–271,1440–1451`
  - Test: `runtime/rule/thread_primitive_test.rs` ·
    `given_named_thread_builder_spawn_with_expectation_when_checked_then_reports_the_spawn`
- [ ] **10A04** — missed · undecided
  - Doc (`no-callbacks.rs.md:275`): "the owner joins the thread when it stops"
  - Gap: a dropped `JoinHandle` of `std::thread::spawn` is accepted; nothing checks the join.
  - Rule: `DetachedTask` or a new one, `runtime/rule/task_visitor.rs:1161–1175`
- [ ] **10A08** — missed · test written
  - Doc (`no-callbacks.rs.md:59–60`): "A loop over a `broadcast::Receiver` matches `Lagged` and goes on.
    `while let Ok(event) = receiver.recv().await` ends at the first lag"
  - Gap: the receiver passed as the closure parameter of `stream::unfold`, the document's own shape, is not
    seen.
  - Rule: `BroadcastLag`, `runtime/rule/typed_runtime_visitor.rs:1065–1076,1166–1181`
  - Test: `runtime/rule/broadcast_lag_test.rs` ·
    `given_broadcast_lag_unfold_closure_receiver_when_linted_then_reports_the_stopping_arm`
- [ ] **10A10** — missed · undecided
  - Doc (`no-callbacks.rs.md:135`): "The contract in `api` declares only `async fn`s, `watch::Receiver`s and
    streams"
  - Gap: a contract method that takes an `mpsc::Sender` as a registration sink for events is accepted.
  - Rule: `ContractShape`, `declaration/util/contract_method_shape.rs:209–233`
- [ ] **7A06** — no rule · closed by decision (F25; the document was then edited to say so)
  - Doc (`no-callbacks.rs.md:15–17`): "It does not flag a closure inside a container field
    (`Vec<Box<dyn Fn…>>`, `HashMap<…, Box<dyn Fn…>>`) …, so that field stays with the reviewer."
  - Gap: a closure inside a container field is not reported. The document now agrees with the code.
- [ ] **seat A row 57** — no rule · not mechanical (the choice of a capacity is semantic)
  - Doc (`no-callbacks.rs.md:53–56`): "a `broadcast` channel for an event several subscribers read. Its
    capacity is a required argument … an `mpsc` channel with an explicit capacity for events one consumer must
    not lose."
- [ ] **seat A row 60** — no rule · not mechanical (the Bad form is a `bool` field in a state struct)
  - Doc (`no-callbacks.rs.md:105–106`): "A one-off event from a state holder to the one consumer that acts on it
    … is an `mpsc` channel, never a flag in the state that the consumer has to clear, and never a `broadcast`"
- [ ] **seat A row 67** — no rule · not mechanical (what is "work that ends" needs a classifier)
  - Doc (`no-callbacks.rs.md:282–283`): "Blocking work that ends (a file read, a blocking client call, a heavy
    computation) runs in `tokio::task::spawn_blocking` and is awaited."
- [ ] **11A04** — missed · candidate (round 11), confidence medium (the `?` half is a judgment call)
  - Doc (`no-callbacks.rs.md:59–60`): "A loop over a `broadcast::Receiver` matches `Lagged` and goes on.
    `while let Ok(event) = receiver.recv().await` ends at the first lag"
  - Gap: `if let Ok(event) = receiver.recv().await { … } else { break; }` and `receiver.recv().await?` inside a
    `loop` end at the first lag and are not reported; `while let`, `match` and `let … else` are.
  - Rule: `BroadcastLag`, `runtime/rule/typed_runtime_visitor.rs:1078–1091,1491–1499`
- [ ] **12A03** — missed · candidate (round 12), confidence medium
  - Doc (`no-callbacks.rs.md:286–287`): "An `async fn` never blocks its worker thread: `tokio::time::sleep`, not
    `std::thread::sleep`; `tokio::fs` or `spawn_blocking`, not `std::fs`"
  - Gap: only three read methods of `std::fs::File` are known; `write_all`, `flush` and `sync_all` on a `File`
    in an `async fn` pass.
  - Rule: `BlockingCallInAsync`, `runtime/rule/typed_runtime_visitor.rs:454–468,1417–1475`
- [ ] **12A04** — missed · candidate (round 12), confidence low to medium (the outside-test half of 9S03)
  - Doc (`no-callbacks.rs.md:294`): "A runtime is built and blocked on only in `main` of the entry-point crate"
  - Gap: `futures::executor::block_on` is the only foreign executor named; `pollster`, `async_std`, `smol`,
    `futures_lite`, `tokio_test` and `LocalSet::block_on` pass outside `main`.
  - Rule: `BlockOnOutsideBoundary`, `runtime/rule/runtime_visitor.rs:253–268`
- [ ] **12A05** — missed · candidate (round 12), confidence low
  - Doc (`no-callbacks.rs.md:14–15`): "The checker judges a field of a closure type in a struct and in an enum
    variant alike."
  - Gap: a closure field is read only under `Option`, `Box`, `Arc`, `Rc` or `Pin`; one closure behind a lock or
    a cell (`Mutex<Option<Box<dyn FnOnce()>>>`, `RefCell<…>`) passes, and it is not a container of closures.
  - Rule: `CallbackDeclaration`, `declaration/util/callback_field_type.rs:81–104`

## no-print.rs.md

- [ ] **10A01** — missed · undecided (the bound about workspace methods)
  - Doc (`no-print.rs.md:6–7`): "A caught error is passed to the logger as a field of the event, which keeps its
    `source()` chain, not only formatted into the message."
  - Gap: an error formatted into the message in a `map_err` after a call the linter cannot type
    (`std::fs::File::open`) is accepted.
  - Rule: `ErrorLogging`, `runtime/rule/error_logging.rs:120–185,561–586`

## no-tuples.rs.md

- [ ] **9S06** — missed · test written
  - Doc (`no-tuples.rs.md:5–9`): "Inside a function body a tuple is destructured where it appears … it is never
    bound to one name (`let parts = text.split_at(index);`)."
  - Gap: `for pair in text.char_indices()` and iteration over a typed map bound to one name are not seen as
    tuples (only `zip` is).
  - Rule: `AnonymousTuple`, `tuple/rule/tuple_binding_visitor.rs:248–275,355–375`
  - Test: `tuple/rule/tuple_binding_visitor_test.rs` ·
    `given_char_indices_and_map_iteration_when_item_is_bound_then_requires_destructuring`
- [ ] **10S01** — missed · test written
  - Doc (`no-tuples.rs.md:8–9`): "it is never bound to one name"
  - Gap: a tuple bound to one name through an `if`, a `match` or a block initialiser is not seen.
  - Rule: `AnonymousTuple`, `tuple/rule/tuple_binding_visitor.rs:329–353`
  - Test: `tuple/rule/tuple_binding_visitor_test.rs` ·
    `given_tuple_bound_through_branch_initializer_when_checked_then_requires_destructuring`

## notes.rs.md

- [ ] **10T10** — missed · undecided (Cargo's behaviour is the auditor's recollection, not checked)
  - Doc (`notes.rs.md:3`): "Stay on stable Rust: no nightly-only language feature and no nightly-only `rustfmt`
    or `clippy` option."
  - Gap: `RUSTC_BOOTSTRAP` in `[env]` of `.cargo/config.toml` is accepted.
  - Rule: `LintConfig`, `config/rule/lint_config.rs:531–590`
- [ ] **7T05 (10T07)** — missed · closed by decision (F29: the rule's committed fixtures pin the root file only)
  - Doc (`notes.rs.md:11`): "`Unanalysable`: a limit of the linter, not of the code (a macro it cannot see
    through)"
  - Doc (`imports.rs.md:3`): "Never use wildcard imports"
  - Gap: an item-position macro call in a file module is not an `Unanalysable` site, and the code inside it (a
    glob import in `cfg_if!`, an `unsafe`) is read by no rule.
  - Rule: `Unanalysable`
- [ ] **11T01** — false positive (a rule with no document) · candidate (round 11), confidence medium
  - Doc (`notes.rs.md:3–4`): "If the project pins a toolchain in `rust-toolchain.toml`, use that one."
  - Gap: a pinned stable toolchain below 1.93 (`channel = "1.88.0"`) is reported against a floor that no
    document states; the committed fixture `cases/rules/LintConfig/fail-old-toolchain` pins the floor, and only
    `rules-rust/README.md:235,244` names it.
  - Rule: `LintConfig`, `config/rule/lint_config.rs:451–491,609`
- [ ] **11T05** — no rule · candidate (round 11), confidence low ("unrelated" is semantic)
  - Doc (`notes.rs.md:21`): "One reason covers one item: there is no baseline and no reason shared by unrelated
    items."
  - Gap: only a missing or blank reason is rejected; the same reason string on any number of `expect`
    attributes or manifest subjects passes.
  - Rule: none; nearest `suppression/` (`expectation_parser.rs:57–68`, `expectation_validate.rs:54–101`)

## region.md

- [ ] **9S01** — missed · test written
  - Doc (`region.md:19`): "Do not add decorative comments, separator comments, or "section header" comments."
  - Gap: a header framed in `~`, `_`, `+` or `/` (`// ~~~ Helpers ~~~`) is accepted; the header character set is
    `-`, `=` and `#` only.
  - Rule: `ForbiddenComment`, `comment/util/comment_shape.rs:2–3,41–51`
  - Test: `comment/rule/forbidden_comment_test.rs` ·
    `given_headers_framed_in_other_delimiters_when_checked_then_reports_each_header_not_prose`

## rustdoc.rs.md

- [ ] **9S02** — missed · undecided
  - Doc (`rustdoc.rs.md:65`): "Comments on trivial getters, structs with self-explanatory fields, or trait impls
    that don't change the trait's contract."
  - Gap: a doc comment on a trivial getter of a published library is not reported; no rule covers the getter
    part.
  - Rule: `DocCommentScope`, `comment/rule/doc_scope_visitor.rs:178–197`
- [ ] **9S08** — missed · test written
  - Doc (`rustdoc.rs.md:68–69`): "`#[doc(hidden)]` anywhere but on an item one crate of the library needs from
    another and no consumer may use"
  - Gap: `#[doc(hidden)] pub` on an item of a published library is never inspected.
  - Rule: `DocHiddenUse`, `module/rule/doc_hidden_visitor.rs:58–61`, `doc_hidden_use.rs:39–65`
  - Test: `module/rule/workspace_visibility_test.rs` ·
    `given_unused_public_hidden_function_when_published_then_reports_the_item`
- [ ] **10S05** — missed · undecided
  - Doc (`rustdoc.rs.md:56`): "Items are linked with intra-doc links …, never named in plain backticks where a
    link resolves."
  - Gap: a backticked function name with parentheses (`` `compute_total()` ``) is not treated as a link
    candidate.
  - Rule: `IntraDocLink`, `comment/util/doc_inline_code.rs:29–72`
- [ ] **11S01** — no rule · candidate (round 11), confidence medium (the exception "a reason that points outside
  the code" is the reviewer's call, so only a partial rule is possible)
  - Doc (`rustdoc.rs.md:9`): "In a plugin, an application or a service the code is understandable without doc
    comments, and none is written."
  - Gap: a plain prose doc comment on an item of an unpublished crate is accepted whatever it says; a non-`fn`
    item is never reported, and a `fn` only for repeating its name or a trait's doc.
  - Rule: `DocCommentScope`, `comment/rule/doc_scope_visitor.rs:47–78,151–227`
- [ ] **11S05** — missed · candidate (round 11), confidence low (the document gives examples, not a list)
  - Doc (`rustdoc.rs.md:62`): "Filler and signature restatement: "This function is responsible for...", "This
    method allows you to...""
  - Gap: the eight filler phrases all begin with "this function" or "this method"; "This struct is responsible
    for …" and "This trait allows you to …" pass.
  - Rule: `RustdocFiller`, `comment/rule/rustdoc_filler.rs:13–35`
- [ ] **12S07** — missed · candidate (round 12), confidence low (the rule's test file was not opened)
  - Doc (`rustdoc.rs.md:5`): "`# Panics` is documented when a function of a published library panics on input or
    state its caller controls"
  - Gap: `unwrap_err` and `expect_err` on a parameter, and a `panic!` inside an `unwrap_or_else` closure, do not
    make `# Panics` required.
  - Rule: `DocCommentScope`, `comment/rule/panic_doc_visitor.rs:69–101,420–429`

## testing.rs.md

- [ ] **9S03** — missed · undecided (is the document's list of three exhaustive?)
  - Doc (`testing.rs.md:35–36`): "An async test is an `async fn` under `#[tokio::test]`, never a runtime the
    test builds or blocks on (`Runtime::block_on`, `Handle::block_on`, `futures::executor::block_on`)."
  - Gap: `tokio_test::block_on`, `pollster::block_on` and `futures_lite::future::block_on` in a test are not
    reported.
  - Rule: `BlockOnInTest`, `runtime/rule/runtime_visitor.rs:257–263`
- [ ] **9S04** — missed · undecided (`abort` is a deliberate consumer outside tests)
  - Doc (`testing.rs.md:41–43`): "A test awaits the `JoinHandle` of every task it starts, or stops the code
    under test through its own shutdown and awaits that."
  - Gap: a test that only `abort()`s the task it spawned is accepted.
  - Rule: `DetachedTask`, `runtime/rule/task_visitor.rs:1596–1598`
- [ ] **10S02** — missed · test written
  - Doc (`testing.rs.md:137`): "the test block is private"
  - Gap: a root test block declared `pub mod test` or `pub(crate) mod test` is accepted.
  - Rule: `RootFileShape`, `module/rule/root_file_shape.rs:149–173`
  - Test: `module/rule/root_file_shape_test.rs` · `given_public_test_block_when_checked_then_only_that_block_reports`
- [ ] **11S02** — missed · candidate (round 11), confidence low to medium (a renamed import is rare)
  - Doc (`testing.rs.md:14`): "Test names use `given_<state>_when_<action>_then_<outcome>`"
  - Gap: a test under a renamed attribute (`use tokio::test as async_test;` then `#[async_test]`) is not seen as
    a test, so its name is not checked; `TestTimePaused` resolves the same import.
  - Rule: `TestFunctionName`, `naming/rule/name_visitor.rs:159–184`
- [ ] **11S04** — no rule · candidate (round 11), confidence low (round 9 gave the clause to the reviewer)
  - Doc (`testing.rs.md:31–32`): "A race needs worker threads:
    `#[tokio::test(flavor = "multi_thread", worker_threads = 2)]`."
  - Gap: nothing reads `worker_threads`; `#[tokio::test(flavor = "multi_thread")]` without it is accepted.
  - Rule: none; nearest `TestTimePaused`, `runtime/rule/test_runtime_visitor.rs:464–487`
- [ ] **12S04** — no rule · candidate (round 12), confidence low (the Frameworks line may be descriptive)
  - Doc (`testing.rs.md:11`): "Prefer fakes over mocks for repositories and gateways when behavior matters."
  - Doc (`testing.rs.md:183–184`): "**Frameworks**: the built-in `#[test]` harness, `#[tokio::test]` with tokio's
    `test-util` feature in `[dev-dependencies]`."
  - Gap: a mocking crate in `[dev-dependencies]` or `#[automock]` on a contract is reported by nothing.
  - Rule: none; nearest `FakePlacement`, `LintConfig`

## time-api.rs.md

- [ ] **10A09** — missed · test written
  - Doc (`time-api.rs.md:37–38`): "`std::time::Instant` is not a moment: … so it is never stored, sent or shown."
  - Gap: an `Instant` sent through a locally created channel (`mpsc::channel::<Instant>(8)`) is accepted.
  - Rule: `TimePrimitive`, `runtime/rule/time_primitive.rs:52–71,315–334`
  - Test: `runtime/rule/time_primitive_test.rs` ·
    `given_instant_sent_through_local_channel_when_checked_then_reports_the_send`
- [ ] **11A10** — missed · candidate (round 11), confidence low (the suffix list is a design choice)
  - Doc (`time-api.rs.md:43–46`): "the constant stays a `Duration`
    (`const CARD_IN: Duration = Duration::from_millis(400);`, never `const CARD_IN_MS: u64 = 400;`)"
  - Gap: the unit suffixes of an integer constant are `_MS`, `_MILLIS`, `_SECS`, `_SECONDS`, `_NANOS`,
    `_MICROS`, `_MINUTES`, `_HOURS`; `_DAYS`, `_SEC`, `_MIN`, `_MILLISECONDS`, `_NS`, `_US` are not reported.
  - Rule: `TimePrimitive`, `runtime/rule/time_primitive.rs:152–159,386–411`
- [ ] **12A06** — missed · candidate (round 12), confidence low to medium
  - Doc (`time-api.rs.md:37–38`): "so it is never stored, sent or shown."
  - Gap: an `Instant` counts as shown only inside a `tracing` or `log` macro; `format!`, `write!` and an
    assertion message pass.
  - Rule: `TimePrimitive`, `runtime/rule/time_primitive.rs:345–354`

## Not in this list

- The 73 clause groups round 9 gave to the reviewer or to an agent's process (meaning of a name, ownership, what
  a context is): the allocation tables of `reaudit-2026-11-10/seat-{t,s,a}.md` list them.
- Cases where the code already follows the document: F65 (a comment between `use` lines is reported), 9S07
  (`it` as a fn-pointer parameter name is reported), 7A04 and F16 (the committed Bad examples pin the reported
  code as a violation).
- The Kotlin linter. Its gaps were last audited on 2026-10-30/31 in `.claude/handoff/lint-audit-matrix.md` and
  `lint-fix-plan.md`; those lists predate later fixes and document changes.
- `notes.rs.md:5` ("`cargo clippy --workspace --all-targets -- -D warnings` runs in CI"): a CI step of the
  consumer repository; the document names no fixed file a rule could read, so round 12 did not count it.
- The rounds did not converge (20, 35, 22, 19, 18, 29, 22 and 26 new cases in rounds 5–12), so more cases
  exist than this file names.
