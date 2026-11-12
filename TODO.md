# TODO: Rust rules the documents state and the linter lacks

What `.claude/docs` says and the Rust linter (`rules-rust`, crate `klibs-rules`) does not check, or checks
wrongly. Sections are named after the document; every item quotes the document's words. Every item here is not
in the code. The cases were read at `master` `7c2d81e`; those of round 14 at `35b8d3b`, those of round 15 at
`a70c4ce`, those of rounds 16–20 at `321713f`.

The cases that are in the code were removed on 2026-11-12: the implemented ones (rounds 9–10, 13, 14 and 15)
and the `document` ones, whose fix was a document change the code already followed. Their items are in
`git show 321713f:TODO.md` (rounds 9–14) and in `decisions.md` F38 with `reaudit-2026-11-11/r15-brainstorm.md`
(round 15). Where an implemented case's bound left part of its Gap out, that part stays here as `<ID>, rest`.

## How to read an item

- **Kind.** `missed`: a violation the linter does not report. `false positive`: conforming code it reports.
  `no rule`: nothing in the linter looks at the requirement.
- **Status.**
  - `kept (round 13)`: the brainstorm of round 13 (`decisions.md` F35) found no mechanical form that reports no
    conforming code, or found that the code already follows the document; no test.
  - `kept (round 14)`: the audit of 2026-11-11 found the case and no rule was written: no checker tells it
    without a classifier, or the decision belongs to the owner (`decisions.md` F37).
  - `candidate (round 15)`: a case found by reading the documents and the rule source, or by a run; no decision
    (`decisions.md`) covers it, and the bound is not set.
  - `candidate (round 16)`: a case found on 2026-11-12 by six independent auditors (a Sonnet and a Haiku pass per
    document bundle, the Haiku passes split by rule list) and one Fable pass over every document, each read from
    source and never run; the orchestrator re-read the cited branch of every item. The status names the models
    that found it. No decision covers it and the bound is not set; `to confirm` and `document conflict` items need
    the owner's reading first.
  - `candidate (round 17)`: the same, from the second pass on 2026-11-12 (three Sonnet seats and ten Haiku passes
    split by rule list, rules read first), checked the same way; `r17/r17-merge.md` records every check, merge and
    drop, and the withdrawal of 16S-H46.
  - `candidate (round 18)`: the third pass on 2026-11-12 (one Sonnet and two Haiku auditors, clause side first),
    checked the same way (`r18/r18-merge.md`).
  - `candidate (round 19)`: the fourth pass (one Sonnet, two Haiku), aimed at what round 18 left unread
    (`r19/r19-merge.md`).
  - `candidate (round 20)`: the fifth and last pass of 2026-11-12 (one Sonnet, two Haiku; `r20/r20-merge.md`). The
    user stopped the loop here; the rounds did not reach zero (63, 51, 4, 15, 6).
  - `not mechanical (round 15)`: the round 15 audit found the requirement with no carrier and judged that a
    source checker cannot tell it.
  - `not taken (round N)`: the part of an implemented case that its bound (round 13: F32, F33; round 14: F37;
    round 15: F38) named as left out or not decided; the implemented part has its test.
  - `closed by decision`: a recorded decision (`F..` in `decisions.md`) left it out of the linter, and round 13
    confirmed it.
  - `not mechanical`: the audit found no rule and judged that a checker cannot tell the case without a
    classifier, and round 13 confirmed it.
- **Doc** lines are those of `.claude/docs` on 2026-11-10, before the document changes of round 13. **Rule**
  paths are under `rules-rust/modules/rules/src/` and their line numbers are those of `7c2d81e` (round 14:
  `35b8d3b`, round 15: `a70c4ce`, round 16: `321713f`; round 16 cites today's document lines); **Test** paths
  are under `rules-rust/modules/rules/test/`.
- A `Near` line names the nearest item and how the mechanism differs; it may name a removed item.

Sources, under `.claude/reports/rust-linter-research/`: the ledger `uncovered-rust-doc-cases.md`,
`reaudit-2026-11-10/seat-{t,s,a}.md`, `r10-seat-{t,s,a}.md`, `r11-seat-{t,s,a}.md` and `r12-seat-{t,s,a}.md`
(the write-up of each case under `#### <ID>`), `reaudit-2026-11-10/owner-questions-brainstorm-r13.md` and
`owner-questions-brainstorm-r13-9s08.md` (the bound of each round 13 case), `decisions.md` (F1–F38),
`reaudit-2026-11-11/r14-seat-{t,s,a}.md` (the round 14 candidates, each under `#### 14<letter><nn>`, with its
Continuation section), `reaudit-2026-11-11/todo-verification.md` (the check of this list against the code) and
`reaudit-2026-11-11/r15-brainstorm.md` (the round 15 bounds), and `reaudit-2026-11-12/r16/` (the round 16 brief,
the seat reports `r16-seat-{t,s,a}-{sonnet,haiku}*.md`, `r16-fable.md`, and `r16-merge.md`, which records the check
and the merge or drop of every candidate), `reaudit-2026-11-12/r17/` to `r20/` (the same for rounds 17–20).

Totals: 194 items. 16 `kept (round 13)`, 6 `kept (round 14)`, 2 `candidate (round 15)`, 1 `not mechanical (round
15)`, 63 `candidate (round 16)` (52 missed, 8 false positive, 3 not mechanical), 51 `candidate (round 17)` (35
missed, 11 false positive, 5 not mechanical), 4 `candidate (round 18)` (2 missed, 1 false positive, 1 not
mechanical), 15 `candidate (round 19)` (11 missed, 4 false positive), 6 `candidate (round 20)` (4 missed, 1 no
rule, 1 false positive), 17 `not taken` (two of them join the rests of two cases), 4 `closed
by decision`, 9 `not mechanical`.

## arch-context.md

- [ ] **10T11** — false positive (document conflict) · kept (round 13): the app `api` list of
  `arch-package.md:54–56` has no `memory`; `arch-context.md:8–10` covers a plugin's `api`
  - Doc (`arch-context.md:8–10`): "a use of an `api` unit or of the storage contracts unit (its contracts,
    models, model rules in `util`, use cases, and its `stub` and `memory` implementations)"
  - Doc (`arch-package.md:55–56`): "a feature others depend on is an `api` unit (contracts, the models in their
    signatures, the `util` rules of those models, stubs)"
  - Gap: `KindShape` rejects a `memory` kind in an app `api` crate; one document names it as `api` content and
    the other does not.
  - Rule: `KindShape`, `module/rule/kind_shape.rs:127–129`
- [ ] **10T04** — missed · kept (round 13): a `<Type>Factory` sits in the file of the type it builds
  (`arch-domain.rs.md:167–170`), in any kind
  - Doc (`arch-context.md:91–93`): "permits a nested `fun interface Factory` in an `api` contract (in Rust, a
    `<Type>Factory` trait in the contract's file, `arch-domain.rs.md`)"
  - Doc (`arch-package.md:200`): "A domain contract sits in the `api` package of its concept."
  - Gap: a `<Type>Factory` trait is accepted in any kind, beside a struct or enum that is not a contract.
  - Rule: `ContractPlacement`, `module/rule/contract_placement.rs:225–235`
- [ ] **14T03** — not mechanical · kept (round 14): a router's implementation may import the UI kinds it
  opens; the documents name a router by what it does, and no path, name or attribute tells one to a checker
  - Doc (`arch-context.md:71`): "a router's implementation also the UI kinds it opens"
  - Gap: an implementation in a technology module is allowed `api`, `model`, `util` and its own technology
    only; the router row's extra allowance (the UI kinds it opens) has no carrier, and a router is told by
    behaviour, not by a word the linter knows.
  - Rule: `KindImportDirection`, `module/rule/kind_import_direction.rs:146–173`
  - Near: **10T11** — that item is `memory` in an app `api`; no item concerns the router row.

- [ ] **16T-H20** — missed · candidate (round 16; Haiku): a UI kind may import a fixed entry-point kind
  - Doc (`arch-context.md:71`): "the other UI kinds of the same unit, with no cycle between them"
  - Gap: a non-fixed entry kind (`gui`, `slot`, `component`, …) is allowed every entry kind, `command` and `event`
    included; the document allows a UI kind only the other UI kinds.
  - Rule: `KindImportDirection`, `module/rule/kind_import_direction.rs:255–259`

- [ ] **17T-H12** — missed · candidate (round 17; Haiku): another host's core-platform unit
  - Doc (`arch-context.md:100–101`): "`core` and its platform unit (`core/bukkit`)"
  - Gap: a platform or unsplit context unit may depend on any `core-platform` unit, another host's included.
  - Rule: `UnitDependency`, `dependency/util/role_dependency.rs:83–92`

- [ ] **19T-S03** — missed (to confirm) · candidate (round 19; Sonnet): `impl-ui` depending on a sibling `impl`
  - Doc (`arch-context.md:23`): "In an app, only the entry-point unit depends on an `impl` unit."
  - Gap: under the default `implementation-access = "siblings"` an `impl-ui` crate may depend on a sibling `impl`.
  - Rule: `UnitDependency`, `dependency/util/role_dependency.rs:39–48`; default at `config/util/config_parse.rs:191–195`

## arch-domain.md

- [ ] **T18** — missed · closed by decision (F26: not reported, as in the Kotlin rule), confirmed in round 13
  - Doc (`arch-domain.md:19–21`): "It is never a member of the type …, and never a top-level function that
    takes the type as a parameter."
  - Gap: a free `util` function taking one model plus a non-model parameter
    (`fn is_expired(invoice: Invoice, now: SystemTime) -> bool`) is accepted; only one-parameter functions are
    checked.
  - Rule: `UtilFunctionAsExtension`, `module/rule/util_function_as_extension.rs:18–23`

- [ ] **16S-S06** — not mechanical · candidate (round 16; Sonnet)
  - Doc (`arch-domain.md:47`): "An entity gets no interface."
  - Gap: nothing links a trait in `api` to a model struct with a typed identifier; which trait is "the entity's
    interface" has no fixed shape.
  - Rule: none; nearest `ContractShape`
- [ ] **16S-S08** — missed · candidate (round 16; Sonnet, Haiku): identifiers held in a collection or under
  another name
  - Doc (`arch-domain.md:37–38`): "never a bare string or number"
  - Gap: only fields named `id` or `*_id` are read; `member_ids: Vec<Uuid>`, `HashSet<String>` of ids and a field
    `uuid: Uuid` pass.
  - Rule: `BareIdentifier`, `declaration/rule/bare_identifier.rs:109–118,150–156`

- [ ] **17S-H33** — false positive (to confirm) · candidate (round 17; Haiku): every `*Validator`
  - Doc (`arch-domain.md:147–150`): "There is no `validation` package and no validator class that duplicates a model
    invariant."
  - Gap: every type named `*Validator` is reported, an entry point's input-shape check included.
  - Rule: ` `, `naming/rule/roleless_type_name.rs:200`

- [ ] **20S-H02** — missed (to confirm) · candidate (round 20; Haiku): a model behind a smart pointer
  - Doc (`arch-domain.md:19–21`): "never a top-level function that takes the type as a parameter"
  - Gap: the outer type is resolved without its generic arguments; `fn total(invoice: Arc<Invoice>)` (or `Box`, `Rc`)
    in `util` passes.
  - Rule: `UtilFunctionAsExtension`, `source/syn/workspace_type_resolver.rs:181–183`

## arch-domain.rs.md

- [ ] **12S03** — missed · kept (round 13): `ModelContainsFunctions` already reports a `new` without `assert!`
  in a model file
  - Doc (`arch-domain.rs.md:61–62`): "A value object with an invariant keeps its fields private … Its `new`
    checks the invariant"
  - Gap: a `new` that guards with `if … { panic!() }` instead of `assert!` is not recognised as an invariant
    check, so the type's `pub` field passes; a test pins this form as unreported.
  - Rule: `ValueObjectShape`, `declaration/rule/value_object_shape.rs:65–68,139–176`
- See also **10S06** under `code-ordering.rs.md` (`arch-domain.rs.md:100–101`: "A struct that only its variants
  carry sits in its file.").

- [ ] **14S09** — false positive (document conflict) · kept (round 14): a `&mut self` method of a std
  operator or library trait on a model type is reported; the rule follows the sentence at `:19–20`, and which
  of the two sentences gives way is the owner's call, in the document
  - Doc (`arch-domain.rs.md:13–15,19–20`): "an operator of `std::ops` … any trait a library's API requires"
    against "No method of a model type takes `&mut self`"
  - Gap: a trait impl whose trait signature takes `&mut self` (`AddAssign`, `Iterator`) on a model type is
    reported as a mutable transition; the document allows the trait and forbids the receiver.
  - Rule: `ModelShouldBeImmutable`, `module/rule/model_should_be_immutable.rs:103–108`
  - Near: **12S05** — that item bounds the receiver type to `Pin<&mut Self>`; the trait-impl conflict is not
    named.

- [ ] **15S01, rest** — missed · not taken (round 15): with `platform-words` empty, any derive outside `std`,
  `core` and `alloc` or any `impl` of an external trait still exempts an event in the `event` kind of a
  `platform` crate
  - Doc (`arch-domain.rs.md:171–175`): "A domain event is a `struct` in `model` named in the past tense … A
    platform's event type that carries it, declared only for other programs, lives in the `event` kind of the
    platform unit."
  - Gap: `#[derive(Serialize)] pub(crate) struct PlayerJailedEvent { … }` in the `event` kind of a `platform`
    crate passes when `platform-words` is empty; with the list set, only a platform identity exempts (F38).
  - Rule: `EventPlacement`, `module/rule/event_placement.rs` (`is_host_derive_path`, `is_host_trait_path`)
- [ ] **15S03** — missed · candidate (round 15, confirmed by a run on 2026-11-12): with `platform-words` empty, a
  derive from another workspace crate exempts a domain event in the `event` kind of a `platform` crate, while an
  `impl` of a trait from the same crate does not
  - Doc (`arch-domain.rs.md:174–175`): "A platform's event type that carries it, declared only for other programs,
    lives in the `event` kind of the platform unit."
  - Gap: `is_host_derive_path` falls back to `is_foreign_path`, true for any package but `std`, `core` and
    `alloc`, the workspace's own crates included; `external_trait` drops a trait that resolves into the workspace.
    The test `given_no_platform_words_when_event_uses_derive_or_trait_of_another_workspace_crate_then_reports_both`
    (uncommitted) expects lines 3 and 5 and gets 5 only. With `platform-words` set both are reported
    (`given_platform_words_when_event_uses_derive_or_trait_of_another_workspace_crate_then_reports_both` passes).
  - Rule: `EventPlacement`, `module/rule/event_placement.rs` (`is_host_derive_path`, `is_foreign_path`)
  - Near: **15S01, rest** — that item is the third-party half of the empty-list exemption; this is the
    workspace half.

- [ ] **16S-S07** — missed · candidate (round 16; Sonnet): a model field changed through a mutator method
  - Doc (`arch-domain.rs.md:19–21`): "no code assigns a field of a model value"
  - Gap: `self.slot.take()`, `replace`, `get_or_insert` on an `Option` field and `std::mem::replace`/`swap`/`take`
    of a field are not field writes; only `=`, compound assignment and collection mutators on collection fields are.
  - Rule: `ModelShouldBeImmutable`, `module/rule/model_mutation_visitor.rs:376–431`
- [ ] **16S-H20** — missed · candidate (round 16; Haiku): a model field written through a borrow the visitor
  cannot target
  - Doc (`arch-domain.rs.md:19–21`): "no code assigns a field of a model value"
  - Gap: `targets` returns nothing for a method call or an index expression, so a write reached through
    `get_mut(..)`, `&mut items[i]` or `iter_mut().next()` passes.
  - Rule: `ModelShouldBeImmutable`, `module/rule/model_mutation_visitor.rs:288–310`
  - Near: **16S-S07** — mutator methods; this is the write target.
- [ ] **16S-H21** — false positive (to confirm) · candidate (round 16; Haiku): the prescribed aggregate factory
  taking a model
  - Doc (`arch-domain.rs.md:166–167`): "An aggregate factory with rules is a free function `create_<aggregate>(…)
    -> Result<Aggregate, AggregateError>` in the aggregate's `util`"
  - Gap: `create_invoice(draft: InvoiceDraft) -> Result<Invoice, InvoiceError>` in `util` is reported as a
    one-model rule because its one parameter is a model.
  - Rule: `UtilFunctionAsExtension`, `module/rule/util_function_as_extension.rs:50–65`
- [ ] **16T-H24** — false positive (document conflict) · candidate (round 16; Haiku): a `pub` platform event type
  - Doc (`arch-domain.rs.md:174–175`): "A platform's event type that carries it, declared only for other programs,
    lives in the `event` kind of the platform unit."
  - Doc (`arch-package.rs.md:135–136`): "Entry points (`InvoiceCommand`, `InvoiceListener`) are `pub(crate)`"
  - Gap: an event type other programs read must be `pub`, and every `pub` type in an entry kind is reported; which
    sentence gives way is the owner's call.
  - Rule: `EntryPointVisibility`, `module/rule/entry_point_visibility.rs:11–15`
- [ ] **16T-H40** — missed · candidate (round 16; Haiku): any prelude trait counts as a conversion
  - Doc (`arch-domain.rs.md:181–182`): "converts to and from the model at its own edge, by `From` or `TryFrom` impls
    in its own file"
  - Gap: a single-segment trait path that resolves `Local` is a "standard conversion", and every builtin name
    (`Into`, `TryInto`, `Default`, `Clone`, `Iterator`, …) resolves `Local`; `impl Into<Row> for Invoice` in a
    technology file passes.
  - Rule: `ImplPlacement`, `module/rule/impl_placement.rs:39–42`; `source/util/crate_index_ext.rs:169–215`
- [ ] **16F-01** — missed · candidate (round 16; Fable): a contract method returning a future type
  - Doc (`arch-domain.rs.md:147–148`): "Its methods are `async fn` where they do I/O, and the trait and every `impl`
    of it carry `#[async_trait]`"
  - Gap: `fn load(&self) -> BoxFuture<'static, Invoice>` (or `Pin<Box<dyn Future …>>`, `impl Future`) passes; a
    trait with no `async fn` needs no `#[async_trait]`, and no return check names a future.
  - Rule: `ContractShape`, `declaration/rule/contract_shape.rs:16–20,76–77`; `declaration/util/contract_method_shape.rs:125–182`
- [ ] **16F-02** — missed · candidate (round 16; Fable): a concrete stream returned under a name other than
  `events`
  - Doc (`arch-domain.rs.md:172–174`): "hands it out as a stream …: `fn events(&self) -> BoxStream<'static,
    InvoiceEvent>`"
  - Gap: the `BoxStream<'static, _>` check runs for a method named `events` only; `BroadcastStream<T>` or
    `ReceiverStream<T>` returned by another method passes.
  - Rule: `ContractShape`, `declaration/util/contract_method_shape.rs:154–177,314–315`
- [ ] **16F-03** — false positive (document conflict) · candidate (round 16; Fable): the framework's error `impl`
  has no legal place
  - Doc (`arch-domain.rs.md:177–179`): "gets that trait's `impl` in the `error` kind of its entry-point unit"
  - Doc (`arch-package.rs.md:256`): "The root of the entry-point crate holds its `di` block and the items the
    platform calls, nothing else."
  - Gap: the `error`-kind exemption needs an `entry-point` crate, whose root admits no `error` or `model` block.
  - Rule: `ImplPlacement`, `module/rule/impl_placement.rs:163–172`; `EntryPointShape`, `module/rule/entry_point_shape.rs:76–91`

- [ ] **17S-S04** — missed · candidate (round 17; Sonnet): an identifier as a type alias
  - Doc (`arch-domain.rs.md:57–58`): "A typed identifier is a tuple struct over the primitive"
  - Gap: only structs and enums are read; `pub(crate) type InvoiceId = Uuid;` in `model` passes.
  - Rule: `BareIdentifier`, `declaration/rule/bare_identifier.rs:129–146`
- [ ] **17S-H32** — false positive (to confirm) · candidate (round 17; Haiku): an entity with one field
  - Doc (`arch-domain.rs.md:58–59`): "A single-property value object is a tuple struct too."
  - Gap: every one-field named struct in `model` is reported, an entity `Player { id: PlayerId }` included.
  - Rule: `BareIdentifier`, `declaration/rule/bare_identifier.rs:131–137`
- [ ] **17S-H53** — false positive (to confirm) · candidate (round 17; Haiku): a generic contract
  - Doc (`arch-domain.rs.md:146`): "A repository, like every contract in `api`, is a trait with the supertraits
    `Send + Sync`"
  - Gap: any generic parameter or `where` clause on an `api` trait is reported; the document names no such limit
    (a test pins the behaviour).
  - Rule: `ContractShape`, `declaration/rule/contract_shape.rs:70–75`
- [ ] **17A-H33** — false positive (minor, to confirm) · candidate (round 17; Haiku): a `'static` supertrait bound
  - Doc (`arch-domain.rs.md:146`): "a trait with the supertraits `Send + Sync`"
  - Gap: a lifetime bound gives `None` and makes the trait invalid; `trait InvoiceRepository: Send + Sync + 'static`
    is reported.
  - Rule: `ContractShape`, `declaration/rule/contract_shape.rs:31–56`
- [ ] **17S-H54** — not mechanical · candidate (round 17; Haiku): an aggregate factory under another name
  - Doc (`arch-domain.rs.md:166–167`): "An aggregate factory with rules is a free function `create_<aggregate>(…)`"
  - Gap: a factory not named `create_*` cannot be told from another function returning the aggregate.
  - Rule: `AssociatedFactoryFunction`
- [ ] **17S-H11** — not mechanical · candidate (round 17; Haiku): a domain event without the `Event` suffix
  - Doc (`arch-domain.rs.md:171`): "A domain event is a `struct` in `model` named in the past tense (`InvoicePaid`)"
  - Gap: events are selected by the `Event` suffix; `PlayerJailed` outside `model` passes, and a past-tense name
    needs a classifier.
  - Rule: `EventPlacement`, `module/rule/event_placement.rs:70`

- [ ] **19S-H01** — missed · candidate (round 19; Haiku): a value object field built from an expression
  - Doc (`arch-domain.rs.md:61–65`): "each field gets an accessor named after what it holds"
  - Gap: a tuple field is named from the constructor argument; `Self(cents * 100)` gives no name, so no accessor is
    required.
  - Rule: `ValueObjectShape`, `declaration/rule/value_object_shape.rs:238–256`
- [ ] **19S-H06** — missed (to confirm) · candidate (round 19; Haiku): an aggregate factory named `new`
  - Doc (`arch-domain.rs.md:166–167`): "never an associated function of the model type"
  - Gap: every `new` is exempt; `impl Invoice { fn new(draft: InvoiceDraft) -> Result<Invoice, InvoiceError> }`
    passes, while a value object's `new` returns `Self`.
  - Rule: `AssociatedFactoryFunction`, `module/rule/associated_factory_function.rs:128`

## arch-package.md

- [ ] **12T02** — missed · kept (round 13): `status` and `analysis` are singular; the document leaves plurality
  to the reviewer
  - Doc (`arch-package.md:178–179`): "Every package name is singular, except an inherently plural concept
    (`news`); which concept is plural is the reviewer's call."
  - Gap: every name ending in `us` or `is` is taken as singular, so `menus`, `wikis` and `taxis` pass.
  - Rule: `PluralPackageName`, `module/rule/plural_package_name.rs:13–14`
- See also **10T04** and **10T11** under `arch-context.md` (`arch-package.md:200`, `:55–56`).

- [ ] **16T-S05** — missed · candidate (round 16; Sonnet, Haiku, Fable): what a kind block holds
  - Doc (`arch-package.md:132–136`): "`api` | contracts of the concept: interfaces other files or units call";
    "`usecase` | operations named `<Verb><Object>UseCase`"; "`util` | the level's functions"
  - Gap: no rule checks which items a kind holds beyond traits (`ContractPlacement`) and `util` (`UtilShape`): a
    plain struct or enum in `api`, an enum in `usecase`, and a file of free functions in `model` or `api` pass
    (`OneTypeFamilyPerFile` returns early on a file with no type). Merges 16T-H01 and 16F-09.
  - Rule: none; nearest `ContractPlacement`, `family/rule/one_type_family_per_file.rs:200–202`
- [ ] **16T-H62** — missed · candidate (round 16; Haiku): `Impl` as a prefix
  - Doc (`arch-package.md:211`): "`Impl`, the `I` prefix and the `Interface` suffix are forbidden."
  - Gap: `Impl` is checked as a suffix only; `ImplRatingRepository` passes.
  - Rule: `ImplementationName`, `naming/rule/name_visitor.rs:141–144`
- [ ] **16T-H63** — not mechanical · candidate (round 16; Haiku): irregular plurals
  - Doc (`arch-package.md:178–179`): "Every package name is singular, except an inherently plural concept (`news`)"
  - Gap: plurality is read from a trailing `s`; `people` and `children` pass, and telling them needs a word list.
  - Rule: `PluralPackageName`, `module/rule/plural_package_name.rs:25–26`
  - Near: **12T02** — the opposite direction (`-us`, `-is` taken as singular).
- [ ] **16F-10** — missed · candidate (round 16; Fable): an implementation under another concept than its
  contract
  - Doc (`arch-package.md:206–208`): "A contract's implementation sits in the technology's package or unit, under
    the same concept"
  - Gap: no rule compares an implementation's concept with its contract's; `item_concept` has one caller,
    `FakePlacement`.
  - Rule: none; nearest `FakePlacement`, `module/util/placement_inspect.rs:69–83`

- [ ] **17T-H53** — missed · candidate (round 17; Haiku; widened in round 20): a pass-through use case through `?`
  or a wrapper
  - Doc (`arch-package.md:239–241`): "No pass-through … A use case exists only when the flow has more than one step"
  - Gap: the forwarding test knows a bare `self.<field>` call or a UFCS contract call; `let rating =
    self.repository.load(id).await?; Ok(rating)` (no `Expr::Try` arm) and `Some(self.ratings.rating(id))` (a
    wrapper `Call` arm, `:209–212`) are not pass-throughs.
  - Rule: `UseCaseShape`, `module/rule/use_case_shape.rs:29–42,168–215`
- [ ] **17T-H54** — missed · candidate (round 17; Haiku): a use-case implementation with a vague prefix
  - Doc (`arch-package.md:206–213`): "`local` and `remote` are not technology names … an app's use case … is named
    `Default<Interface>`"
  - Gap: only traits in the `api` kind are checked; a use-case trait in the `usecase` kind of its `api` unit lets
    `LocalAddRatingUseCase` pass.
  - Rule: `ImplementationName`, `naming/rule/implementation_name.rs:29–31`
- [ ] **17T-H55** — missed (weak) · candidate (round 17; Haiku; widened in round 18 by Sonnet): plural roleless
  names
  - Doc (`arch-package.md:263–265`): "there is no `Helper`, `Util` or `Manager` class"
  - Gap: the words are matched as suffixes; `InvoiceHelpers` and `InvoiceManagers` pass, and `Service` is dropped
    when a lowercase letter follows (`InvoiceServices`).
  - Rule: `KindNameInPackage`, `naming/rule/name_visitor.rs:131–138,165–167`
- [ ] **17T-H58** — not mechanical · candidate (round 17; Haiku): the verb of a use case name
  - Doc (`arch-package.md:135`): "`usecase` | operations named `<Verb><Object>UseCase`"
  - Gap: whether the first word is a verb needs a word list.
  - Rule: `UseCaseShape`
- [ ] **17T-H71** — missed (weak) · candidate (round 17; Haiku): `etc.` at the end of a README sentence
  - Doc (`arch-package.md:77–78`): "Every unit carries a `README.md` of one sentence stating its responsibility."
  - Gap: `etc.` and `vs.` are removed with their period, so two sentences after `etc.` count as one.
  - Rule: `UnitReadme`, `readme/util/sentence_count.rs:6–28`
- [ ] **18S-S05** — not mechanical · candidate (round 18; Sonnet): a function file named after its decision
  - Doc (`arch-package.rs.md:290–291`): "A file of free functions is named …" (after the decision it makes)
  - Gap: only generic names (`utils`, `helpers`, `extensions`, `_ext`) are rejected; whether a name states the
    decision needs a word list.
  - Rule: `UtilFileName`, `module/rule/util_file_name.rs:56–66,231`

## arch-package.rs.md

- [ ] **10T05** — missed · kept (round 13): without a target `cfg` a module named `windows` is not platform
  code; the Kotlin rule has the same guard
  - Doc (`arch-package.rs.md:83–84`): "Platform code … is code under a target `cfg` … at the same module path,
    never in a module named after the platform."
  - Gap: a module named after a platform (`linux`) is accepted in a crate that has no target `cfg`.
  - Rule: `PlatformPackage`, `module/rule/platform_package.rs:64–66`
- [ ] **7T06, package-key half** — missed · closed by decision (F28: about 80 committed fixtures and the clean
  consumer fixture are laid out that way), confirmed in round 13
  - Doc (`arch-package.rs.md:219–220`): "Shared settings are declared once in the root: `[workspace.package]`,
    `[workspace.dependencies]` and `[workspace.lints]`"
  - Gap: a package key (`edition`, `license`, …) written with the same literal in two members while the root
    declares none is accepted. The dependency half of 7T06 is fixed.
  - Rule: `RootBuildSubprojects`, `layout/rule/root_build_subprojects.rs`
- [ ] **12T01, rest** — missed · not taken (round 13): a renamed dependency
  - Doc (`arch-package.rs.md:219–220`): "Shared settings are declared once in the root: `[workspace.package]`,
    `[workspace.dependencies]` and `[workspace.lints]`"
  - Gap: a member's `alias = { package = "foo", version = "1" }` against the root's `foo` escapes; dependencies
    are compared by key. `foo = "1"` against `foo = { version = "1" }` is reported.
  - Rule: `RootBuildSubprojects`, `layout/rule/root_build_subprojects.rs:29–117`
- [ ] **12T04, rest** — missed · not taken (round 13): `#[doc(hidden)] pub` modules, re-exports and macros
  (kept: the name index records no user of them); fields and variants (not decided)
  - Doc (`arch-package.rs.md:147–148`): "Each `#[doc(hidden)] pub` item is used by another crate of the
    workspace; the checker reports one that is not."
  - Gap: only structs, enums, unions, traits, type aliases, functions, constants, statics and members are
    checked; a `#[doc(hidden)] pub` module, re-export, macro, field or variant never is.
  - Rule: `DocHiddenUse`, `module/rule/doc_hidden_use.rs:51–58`
- [ ] **14T06, rest** — false positive · not taken (round 14): a use of a `pub` tuple field through an alias
  of its type
  - Doc (`arch-package.rs.md:131–133`): "Fields and methods take the tier of their widest user the same way:
    private unless another file uses them"
  - Gap: another crate that names the type through a `type` alias or a renamed import before a parenthesis is
    not seen as a user, so the `pub` tuple field it uses is reported.
  - Rule: `VisibilityTier`, `module/rule/visibility_tier.rs` (`check_tier_members`, `is_unused_public`)

- [ ] **14T05** — not mechanical · kept (round 14): build automation outside the workspace root and member
  directories; a script in another directory that runs no `cargo` may be a container entry point or a test
  fixture, and what counts as build automation is the reviewer's call
  - Doc (`arch-package.rs.md:241–243`): "Build automation … is a subcommand of the `xtask` crate …, never a
    shell script."
  - Gap: a shell script in a non-member subdirectory is reported only when it runs `cargo`; a release or
    code-generation script that calls other tools passes, and what is build automation needs a classifier.
  - Rule: `ShellAutomation`, `layout/rule/shell_automation.rs:92–101`
  - Near: **10T14** — that item is the extensionless shebang inside the same directory scope; this is the
    directory scope itself.

- [ ] **16T-S01** — missed (implemented case 7T03, rest) · candidate (round 16; Sonnet): unsafe attributes
  - Doc (`arch-package.rs.md:150–152`): "Unsafe code sits in the technology module of the library or system
    interface it calls"
  - Gap: `#[unsafe(no_mangle)]`, `#[unsafe(export_name)]` and `#[unsafe(link_section)]` on a safe item outside a
    technology module are not visited; 7T03 took the `unsafe extern` half only. Whether such an attribute counts as
    unsafe code is the owner's reading.
  - Rule: `UnsafePlacement`, `module/rule/unsafe_visitor.rs:62–116`
- [ ] **16T-S02** — missed · candidate (round 16; Sonnet): `build.rs`, `examples/` and `benches/` are read by no
  source rule
  - Doc (`imports.rs.md:3`): "Never use wildcard imports"; (`arch-package.rs.md:366`): "`benches/`, `examples/` and
    `build.rs` keep their Cargo meaning."
  - Gap: the source reader keeps only lib, bin and the proc-macro `tests/it` targets, so `WildcardImport`,
    `LocalItem`, `UnsafePlacement` and the other source rules never see those files.
  - Rule: source reader, `source/syn/syn_source_reader.rs:48–60,82–106`
- [ ] **16T-S03** — missed · candidate (round 16; Sonnet): a module named after a target architecture
  - Doc (`arch-package.rs.md:83–84`): "code under a target `cfg` (`target_os`, `target_family`, `unix`, `windows`,
    `target_arch`) at the same module path, never in a module named after the platform"
  - Gap: the fixed word list holds OS and family words only; `mod aarch64`, `mod x86_64` or `mod uefi` passes unless
    the crate's own cfgs name that value.
  - Rule: `PlatformPackage`, `module/rule/platform_package.rs:26–48`; `PlatformSourceSetUnit` shares the list
  - Near: **10T05** — a platform-named module in a crate with no target `cfg`.
- [ ] **16T-S04** — missed · candidate (round 16; Sonnet): a `pub(crate)` tuple field
  - Doc (`arch-package.rs.md:131–132`): "Fields and methods take the tier of their widest user the same way: private
    unless another file uses them"
  - Gap: unnamed fields are dropped before the usage test, so a `pub(crate)` tuple field used only in its own file
    passes.
  - Rule: `VisibilityTier`, `module/rule/visibility_tier.rs:174–181`
  - Near: **14T06, rest** — a false positive through an alias.
- [ ] **16T-H05** — missed · candidate (round 16; Haiku): an enum in `memory`
  - Doc (`arch-package.rs.md:316–318`): "`InMemory<Trait>` in the `memory` module of the concept"
  - Gap: every non-struct item is skipped; an enum implementing a contract in `memory` under another name, or an
    `InMemory*` enum outside `memory`, passes.
  - Rule: `InMemoryPlacement`, `module/rule/in_memory_placement.rs:45–47`
- [ ] **16T-H21** — missed · candidate (round 16; Haiku): a `pub enum` implementation
  - Doc (`arch-package.rs.md:128`): "used by other files of the same crate, such as an implementation its DI module
    constructs | `pub(crate)`"
  - Gap: only structs are read; a `pub enum` in a technology module that implements a crate trait passes.
  - Rule: `ImplementationVisibility`, `module/rule/implementation_visibility.rs:43–44`
- [ ] **16T-H22** — missed · candidate (round 16; Haiku): the fake feature through another crate's feature
  - Doc (`arch-package.rs.md:341–342`): "Another crate enables the feature only in `[dev-dependencies]`, never in
    `[dependencies]`"
  - Gap: `fake-support = ["shop-billing-api/fake"]` in crate B, enabled by a normal dependency from crate C, puts
    the fake in C's normal build; feature expansion drops every `a/b` child, and only the dependent's own `default`
    is walked for `key/fake`.
  - Rule: `UnitDependency`; `workspace/cargo/cargo_workspace_reader.rs:122–154,168–189`
- [ ] **16T-H23** — missed · candidate (round 16; Haiku): one DI module covers every resource of a crate
  - Doc (`arch-package.rs.md:270–271`): "a resource with its own lifecycle has its own DI module in the crate that
    owns it"
  - Gap: the missing-module check returns as soon as the crate has any DI type, so a second resource with its own
    lifecycle and no DI module passes.
  - Rule: `DiPlacement`, `module/rule/di_placement.rs:41–47`
- [ ] **16T-H41** — missed · candidate (round 16; Haiku): an `impl` on a reference type is no behaviour
  - Doc (`arch-package.rs.md:282–283`): "A type with behaviour always has its own file"
  - Gap: the behaviour owner is found only for a path self type, so `impl Display for &Line` leaves `Line` a plain
    data type in its owner's file.
  - Rule: `OneTypeFamilyPerFile`, `family/rule/one_type_family_per_file.rs:67–69`
- [ ] **16T-H42** — missed · candidate (round 16; Haiku): a `#[doc(hidden)] pub` member "used" by any equal name
  - Doc (`arch-package.rs.md:147–148`): "Each `#[doc(hidden)] pub` item is used by another crate of the workspace;
    the checker reports one that is not."
  - Gap: a member counts as used when any other crate holds a member of the same name, so `#[doc(hidden)] pub fn
    len` is used by every `.len()`.
  - Rule: `DocHiddenUse`, `module/rule/doc_hidden_use.rs:186–194`; `module/rule/visibility_tier.rs:206–219`
  - Near: **12T04, rest** — modules, re-exports and macros are not checked.
- [ ] **16T-H43** — missed (weak) · candidate (round 16; Haiku, Fable): a dependency key that hides a standard
  crate
  - Doc (`arch-package.rs.md:176–177`): "The prefix also keeps a crate from hiding a standard crate: a dependency
    named `core` makes `core::fmt` unresolvable (E0433)."
  - Gap: only package names are read; `core = { package = "shop-core", … }` (or `std`, `alloc`, `test`) passes.
    The sentence may be read as rationale only.
  - Rule: `RootPackagePath`, `layout/rule/root_package_path.rs`
  - Near: **12T01, rest** — renamed dependencies in `RootBuildSubprojects`.
- [ ] **16T-H60** — missed · candidate (round 16; Haiku): `cargo` after a shell keyword
  - Doc (`arch-package.rs.md:241–243`): "Build automation … is a subcommand of the `xtask` crate …, never a shell
    script."
  - Gap: the first token of each segment must be `cargo` (after `exec`, `sudo`, `time`, `env`), so
    `if …; then cargo build; fi` and `do cargo …` in a non-member script pass.
  - Rule: `ShellAutomation`, `layout/rule/shell_automation.rs:16–31,85`
  - Near: **14T05** — the directory scope.
- [ ] **16F-06** — missed · candidate (round 16; Fable): other task-runner files
  - Doc (`arch-package.rs.md:241–243`): "never a shell script"
  - Gap: five fixed names and nine extensions are known; `Makefile.toml`, `Taskfile.yml` and `.justfile` pass.
  - Rule: `ShellAutomation`, `layout/rule/shell_automation.rs:9–14,75–84`
- [ ] **16T-H61** — missed (weak) · candidate (round 16; Haiku): a plugin sub-context split with no `api`
  - Doc (`arch-package.md:49`): "A sub-context that is not split is one unit"
  - Gap: only `impl`, `impl-ui` and `core` roles, or a non-allowed sibling of an `api` unit, are reported; two
    `context` units of one sub-context with no `api` pass.
  - Rule: `PluginImplUnit`, `layout/rule/plugin_impl_unit.rs:12–28`
- [ ] **16F-04** — missed · candidate (round 16; Fable): free functions beside a `type` alias
  - Doc (`arch-package.rs.md:285`): "A file that declares a type holds no free function"; (`code-ordering.rs.md:20`):
    "among the free functions of a file that declares no type"
  - Gap: `CalleeBeforeCaller` counts an alias as a type and skips the file; `OneTypeFamilyPerFile` counts no type
    and returns early; neither checks the file.
  - Rule: `CalleeBeforeCaller`, `ordering/rule/callee_before_caller.rs:76–89`; `OneTypeFamilyPerFile`, `:194–202`

- [ ] **17T-S03** — missed · candidate (round 17; Sonnet, Haiku): `default` enabling `fake` through another feature
  - Doc (`arch-package.rs.md:341–342`): "Another crate enables the feature only in `[dev-dependencies]`"
  - Gap: only a literal `"fake"` in `default` is read; `default = ["testing"]`, `testing = ["fake"]` passes.
  - Rule: `FakePlacement`, `module/rule/fake_placement.rs:108–112`
  - Near: **16T-H22** — the route through another crate.
- [ ] **17T-S04** — missed (weak) · candidate (round 17; Sonnet): a platform dependency in `[dependencies]`
  - Doc (`arch-package.rs.md:101`): "A platform's dependency sits in `[target.'cfg(…)'.dependencies]`."
  - Gap: an external crate used only under a target `cfg` but declared in `[dependencies]` is checked by no rule;
    the mechanical form needs every use site under a target cfg.
  - Rule: none; nearest `ApiUnitPlatformDependency`, `dependency/rule/api_unit_platform_dependency.rs:10–26`
- [ ] **17T-S05** — missed · candidate (round 17; Sonnet): a restricted re-export
  - Doc (`arch-package.rs.md:71–73`): "`pub use` for what another crate uses, `pub(crate) use` for the rest of the
    crate."
  - Gap: `pub(super) use`, `pub(self) use` and `pub(in path) use` are neither externally nor crate public and pass.
  - Rule: `ReExportShape`, `module/rule/re_export_shape.rs:28–52`
- [ ] **17T-S06** — missed · candidate (round 17; Sonnet, Haiku): repeated dependencies in other forms
  - Doc (`arch-package.rs.md:219–220`): "Shared settings are declared once in the root"
  - Gap: only a version string or a `{ version }` table counts as a repeat; `{ git, rev }` repeated in two members,
    and `"1"` against `{ version = "1", features = [..] }`, pass.
  - Rule: `RootBuildSubprojects`, `layout/rule/root_build_subprojects.rs:55–70,87–89`
  - Near: **12T01, rest** — a renamed dependency.
- [ ] **17T-H10** — missed · candidate (round 17; Haiku): `file` and `system` are no technology kind
  - Doc (`arch-package.rs.md:109–113`): "A technology module is named after a crate the crate depends on, or `file`,
    `memory`"
  - Gap: `is_technology_kind` reads dependency words only; a `pub` implementation in `file` or `system` passes
    `ImplementationVisibility`, and `model`, `api` or `usecase` may import a `file` module's type.
  - Rule: `ImplementationVisibility`, `module/rule/implementation_visibility.rs:50–54`; `KindImportDirection`,
    `kind_import_direction.rs:190–197`; `module/util/kind_resolve.rs:78–84`
- [ ] **17T-H14** — missed (weak) · candidate (round 17; Haiku): a domain contract of registration shape
  - Doc (`arch-package.md:216–218`): "framework context parameters and no domain data parameters"
  - Gap: any synchronous `&self` → `()` trait with external parameter types (`String` included) counts as a
    registration trait and may sit in an entry kind.
  - Rule: `ContractPlacement`, `module/util/registration_base.rs:88–98`
  - Near: **10T04** — factories.
- [ ] **17T-H30** — missed · candidate (round 17; Haiku): a nested type also used by a free function
  - Doc (`arch-package.rs.md:287–288`): "An enum used only through one type sits in that type's file; otherwise it is
    its own file."
  - Gap: one use outside a type and its impls makes the owner `None`, and `NestedEnum` and `NestedDataStruct` go
    silent.
  - Rule: `NestedEnum`, `NestedDataStruct`, `module/util/type_ownership.rs:93`
- [ ] **17T-H52** — missed · candidate (round 17; Haiku): a dotted script name
  - Doc (`arch-package.rs.md:241–243`): "never a shell script"
  - Gap: a name with a `.` returns before the shebang test; `bootstrap.local` with `#!/bin/sh` passes.
  - Rule: `ShellAutomation`, `layout/rule/shell_automation.rs:57–60`
- [ ] **17T-H70** — missed (to confirm) · candidate (round 17; Haiku): `macro_rules!` in a root block
  - Doc (`arch-package.rs.md:256`): "The root of the entry-point crate holds its `di` block and the items the
    platform calls, nothing else."
  - Gap: every `Item::Macro` in a root module block is accepted, a `macro_rules!` definition included.
  - Rule: `RootFileShape`, `module/rule/root_file_shape.rs:41`
- [ ] **17T-H73** — false positive (minor, to confirm) · candidate (round 17; Haiku): the array form of the alias
  - Doc (`arch-package.rs.md:242`): "`[alias] xtask = "run --package xtask --"`"
  - Gap: only the string form is accepted; `xtask = ["run", "--package", "xtask", "--"]` is reported.
  - Rule: `XtaskAlias`, `layout/rule/xtask_alias.rs:26–29`
- [ ] **17T-H74** — missed (weak) · candidate (round 17; Haiku): a test in an inline module of a fake file
  - Doc (`arch-package.rs.md:305–309`): "A test file is named after the file of the owner … with `_test`"
  - Gap: only top-level `#[test]` items of a file under `fake/` or `memory/` are seen.
  - Rule: `TestMirror`, `module/rule/test_mirror.rs:59–72`
- [ ] **18T-H01** — missed (weak, to confirm) · candidate (round 18; Haiku): a forbidden name as a file module
  - Doc (`arch-package.rs.md:104–105`): "Never a module name, besides that list: `prelude`; …"
  - Gap: the 30 forbidden words (`prelude`, `domain`, `common`, …) are checked only for an inline module; `prelude.rs`
    inside a kind block passes.
  - Rule: `ForbiddenPackageName`, `module/rule/forbidden_package_name.rs:24–28`

- [ ] **19T-S02** — missed (to confirm) · candidate (round 19; Sonnet): a `pub` field on a type that is not `pub`
  - Doc (`arch-package.rs.md:131–133`): "never `pub` on a type that is not `pub`. `unreachable_pub` … does not report
    fields, so the checker does."
  - Gap: members are checked only on a reachable `pub` type. If rustc's `unreachable_pub` does report such a field
    (from memory), the carrier is B and the sentence is to be corrected instead.
  - Rule: `VisibilityTier`, `module/rule/visibility_tier.rs:283–306`
- [ ] **19T-S05** — missed (weak) · candidate (round 19; Sonnet): skipped directories inside a container
  - Doc (`arch-package.rs.md:171–172`): "`README.md` aside: no `Cargo.toml`, no `src/`, no other file"
  - Gap: the walk skips `resources`, `build`, `node_modules` and hidden directories, so their contents in a
    container are never seen.
  - Rule: `ContainerSources`, `workspace/util/directory_skip.rs:1–7`

## async.rs.md

- [ ] **seat A row 25** — no rule · not mechanical, confirmed in round 13 ("started by a request or user action" needs a classifier)
  - Doc (`async.rs.md:79–80`): "A task started by a request or a user action is guarded atomically against a
    second start. A flag in the state such as `is_busy` is what a reader shows, not a lock"
- [ ] **seat A row 29** — no rule · not mechanical, confirmed in round 13 (a project-written type with no fixed form)
  - Doc (`async.rs.md:134–138`): "The scope is a small type written once. … each such type receives an
    `Arc<TaskScope>`."
  - Gap: the shape of `TaskScope` itself is not checked; only its owner is.
- [ ] **seat A row 32** — no rule · not mechanical, confirmed in round 13 (choosing the primitive is semantic)
  - Doc (`async.rs.md:210–212`): "State a reader observes is a `watch` channel … A counter or a flag is an
    atomic. A resource that one task can own … is owned by that task and reached through an `mpsc` channel."
- [ ] **11A03, rest** — missed (document conflict) · not taken (round 13): `on_reload` is not read, and the
  `on_disable` conflict is not decided
  - Doc (`async.rs.md:217–218`): "A synchronous function on a runtime thread (a `Drop`, a synchronous
    `on_disable`) cannot lock it: `blocking_lock` panics there."
  - Gap: `blocking_lock`, `blocking_read` and `blocking_write` in `on_reload` are not reported. Document
    conflict: `di-manual.rs.md:102–106` and `no-callbacks.rs.md:296–297` run `on_disable` after `block_on`, off
    the runtime.
  - Rule: `BlockingLockInSync`, `runtime/rule/typed_runtime_visitor.rs:1268–1310,1455–1461`
- [ ] **11A09, rest** — missed · not taken (round 13): tokio's unstable `task::Builder::spawn`
  - Doc (`async.rs.md:122–123`): "`tokio::spawn` whose `JoinHandle` is dropped (a bare statement, `let _ =`,
    `drop(…)`) detaches the task: nothing stops it, and its panic reaches no one."
  - Gap: a dropped handle of `tokio::task::Builder::new().spawn(..)` is not reported.
  - Rule: `DetachedTask`, `runtime/rule/task_visitor.rs:172–178,277–301`
- [ ] **15A03, rest** — missed · not taken (round 15): I/O in a `send_modify` or `send_if_modified` closure
  through an untyped receiver, `std::thread::sleep`, a client crate or logging
  - Doc (`async.rs.md:41–43`): "The closure of `send_modify` and `send_if_modified` runs exactly once, under the
    channel's write lock. It computes the new value from the current one and does nothing else: no I/O, no lock,
    no other channel."
  - Gap: `write_all` on a receiver the visitor does not type, `std::thread::sleep`, `reqwest::get` and a
    `tracing` event inside the closure pass; the typed `File`, `TcpStream` and `Command` methods and five
    `std::io` functions are reported (F38).
  - Rule: `ImpureUpdateClosure`, `runtime/rule/typed_runtime_visitor.rs` (`check_update`)
- [ ] **15A04** — false positive (to confirm) · candidate (round 15): the `std::fs::`/`std::net::` prefix test
  of a `send_modify` closure takes a pure constructor for I/O
  - Doc (`async.rs.md:41–43`): "It computes the new value from the current one and does nothing else: no I/O, no
    lock, no other channel."
  - Gap: `std::net::SocketAddr::new(ip, port)` or `std::net::Ipv4Addr::new(..)` inside the closure only builds a
    value, yet a free call under either prefix counts as I/O. Confirmed by a run on 2026-11-12: the test
    `given_watch_closure_building_a_net_value_without_io_when_checked_then_reports_only_the_file_read`
    (uncommitted) expects `[8]` and gets `[2, 5, 8]`.
  - Also (2026-11-12): `std::io::stdout()`, `stdin()` and `stderr()` alone are reported as I/O through the exact
    list 15A03 added, though std's constructors only call `OnceLock::get_or_init` (`library/std/src/io/stdio.rs`,
    1.93.0, `stdout` at `:716–721`, `stdin` at `:340–347`); the lock is taken in `write`, `flush` and `lock()`.
    The round 15 bound takes them on purpose, and the same entry is what catches `writeln!(std::io::stdout(), ..)`
    (the macro arguments are read as calls); the test
    `given_std_stdout_handle_in_watch_closure_when_checked_then_reports_handle_macro_write_and_flush` pins today's
    behaviour.
  - Rule: `ImpureUpdateClosure`, `runtime/rule/typed_runtime_visitor.rs` (`check_update`)
  - Near: **15A03, rest** — that item is I/O the check misses; this is the prefix test reporting too much.

- [ ] **16A-S03** — missed · candidate (round 16; Sonnet): an update closure writing through `self`
  - Doc (`async.rs.md:44–46`): "so it is pure and writes nothing it captures"
  - Gap: a captured write needs a bare-path receiver; `self.last_seen.store(..)`, `self.queue.push(..)` and
    `self.slot.take()` in a `fetch_update` closure pass.
  - Rule: `ImpureUpdateClosure`, `runtime/rule/closure_effects.rs:215–219`; `typed_runtime_visitor.rs:867–877`
- [ ] **16A-S04** — missed · candidate (round 16; Sonnet): an update closure reading a channel
  - Doc (`async.rs.md:41–43`): "no I/O, no lock, no other channel."
  - Gap: only the send-side methods count; `other.borrow()`, `try_recv` or `blocking_recv` inside `send_modify`
    pass.
  - Rule: `ImpureUpdateClosure`, `runtime/rule/typed_runtime_visitor.rs:878–879`
- [ ] **16A-S08** — missed · candidate (round 16; Sonnet): a temporary tokio guard outside a statement
  - Doc (`async.rs.md:213–216`): "`tokio::sync::Mutex` is only for a guard that must live across an `.await`."
  - Gap: the temporary-guard check reads a semicolon statement only; `let n = m.lock().await.len();` and a tail
    `m.lock().await.get(..)` pass.
  - Rule: `LockChoice`, `runtime/rule/typed_runtime_visitor.rs:1893–1898`
- [ ] **16A-H11** — missed · candidate (round 16; Haiku): `futures::future::ready` under a timeout
  - Doc (`async.rs.md:9–12`): "never wraps a read that returns at once"
  - Gap: only `std::future::ready` and `core::future::ready` are immediate futures.
  - Rule: `WatchTimeout`, `runtime/rule/typed_runtime_visitor.rs:1403–1416`
- [ ] **16A-H50** — missed · candidate (round 16; Haiku): a `JoinSet` inside a wrapper behind a lock
  - Doc (`async.rs.md:120`): "A `JoinSet` behind a lock instead keeps every finished task until someone calls
    `join_next`"
  - Gap: the lock's inner type is resolved by its outer path only; `Mutex<Option<JoinSet<()>>>` passes.
  - Rule: `JoinSetBehindLock`, `runtime/rule/typed_runtime_visitor.rs:828–832`
- [ ] **16A-H51** — missed · candidate (round 16; Haiku): a cancel-unsafe call on a generic receiver
  - Doc (`async.rs.md:197–202`): "every branch is cancel safe"
  - Gap: the receiver must resolve to a `tokio` `io`/`fs`/`net` path; `write_all` on `S: AsyncWriteExt` in a
    looping `select!` passes.
  - Rule: `SelectCancelSafety`, `runtime/rule/typed_runtime_visitor.rs:908–923`
- [ ] **16A-H52** — missed (weak) · candidate (round 16; Haiku): a cancel-unsafe call one function deep
  - Doc (`async.rs.md:197–202`): "every branch is cancel safe"
  - Gap: a branch awaiting a workspace `async fn` whose body awaits `read_exact` is not followed.
  - Rule: `SelectCancelSafety`, `runtime/rule/typed_runtime_visitor.rs:1765–1794`

- [ ] **17A-H03** — missed (weak) · candidate (round 17; Haiku): a timeout around a workspace `async fn` that awaits
  nothing
  - Doc (`async.rs.md:9–12`): "an `async` block with no `.await` in it"
  - Gap: only the future expression itself is read.
  - Rule: `WatchTimeout`, `runtime/rule/typed_runtime_visitor.rs:1394–1427`
- [ ] **17A-H05** — missed · candidate (round 17; Haiku): a dropped handle returned by a workspace function
  - Doc (`async.rs.md:122–123`): "`tokio::spawn` whose `JoinHandle` is dropped (a bare statement, …) detaches the
    task"
  - Gap: only spawn paths are known; `start_worker();` returning a spawned `JoinHandle` passes.
  - Rule: `DetachedTask`, `runtime/rule/task_visitor.rs:1355–1366`

- [ ] **19A-H01** — missed · candidate (round 19; Haiku): a write by a non-collection method in a critical section
  - Doc (`async.rs.md:232–236`): "A critical section computes first and writes once"
  - Gap: method writes count only on collection-typed fields; `guard.slot.take()` before the final write passes.
  - Rule: `CriticalSectionWrite`, `runtime/rule/guard_writes.rs:136–171`; `runtime/util/guard_type.rs:3–16`
- [ ] **19A-H02** — missed · candidate (round 19; Haiku): a write through a borrow of the guard
  - Doc (`async.rs.md:232–236`): "A critical section computes first and writes once"
  - Gap: an assignment counts only when rooted at the guard's name; `let slot = &mut guard.slot; *slot = …;` passes.
  - Rule: `CriticalSectionWrite`, `runtime/rule/guard_writes.rs:11–20,112–115`
- [ ] **19A-H09** — false positive (minor) · candidate (round 19; Haiku): an `.await` inside a macro argument
  - Doc (`async.rs.md:216`): "`tokio::sync::Mutex` is only for a guard that must live across an `.await`."
  - Gap: the await visitor does not enter macro tokens, so a guard whose crossing `.await` sits in a macro argument
    is reported as never crossing one.
  - Rule: `LockChoice`, `runtime/rule/await_visitor.rs:25–45`

- [ ] **20A-S02** — missed · candidate (round 20; Sonnet): `std::mem::forget` of a handle
  - Doc (`async.rs.md:122–123`): "`tokio::spawn` whose `JoinHandle` is dropped … detaches the task"
  - Gap: any call taking the handle consumes it; `std::mem::forget(handle)` passes (thread handles too).
  - Rule: `DetachedTask`, `runtime/rule/task_visitor.rs:1742–1790`
- [ ] **20A-S04** — missed (weak) · candidate (round 20; Sonnet): other cancel-unsafe reads
  - Doc (`async.rs.md:197–202`): "every branch is cancel safe"
  - Gap: `read_line`, `read_until` and `read_u*` are not in the method list (tokio's cancel safety from memory).
  - Rule: `SelectCancelSafety`, `runtime/rule/typed_runtime_visitor.rs:891–924`

## clean-code.rs.md

- [ ] **17S-S02** — false positive · candidate (round 17; Sonnet, Haiku): operator methods matched by name
  - Doc (`clean-code.rs.md:144–148`): "`a.mul(b).add(c)` is never written"
  - Gap: a fluent builder `Order::new().add(a).add(b)` is a chain of operators, and in a file that imports
    `std::ops::Add`, `ledger.add(entry);` is a lone operator call.
  - Rule: `OperatorBeforeTrailingCall`, `expression/rule/expression_visitor.rs:166–170,240–242`
- [ ] **17S-H50** — false positive · candidate (round 17; Haiku): a field accessor named after a banned verb
  - Doc (`clean-code.rs.md:18–20`): "These keep their names: … a method whose body only returns a field, named after
    that field"
  - Gap: `fn handle(&self) -> u32 { self.handle }` is reported for `handle`.
  - Rule: `FunctionVerb`, `naming/rule/name_visitor.rs:217–227`
- [ ] **17S-H56** — not mechanical · candidate (round 17; Haiku): a keyed `get` on a type that is no collection
  - Doc (`clean-code.rs.md:19–20`): "`get` … as the lookup of a collection type"
  - Gap: a keyed `get` is exempt on any type; telling a collection needs a classifier.
  - Rule: `FunctionVerb`, `naming/rule/name_visitor.rs:217–222`

## closure-naming.rs.md

- [ ] **14S10, rest** — missed · not taken (round 14): crossbeam's `select!` grammar
  - Doc (`closure-naming.rs.md:6–7`): "Never name anything `it`: not a closure parameter, not a function
    parameter, not a `let` or `match` binding"
  - Gap: an `it` bound in crossbeam's `recv(r) -> it => …` branch is not visited; the tokio and futures branch
    grammar is.
  - Rule: `ExplicitItName`, `ClosureParameterName`, `naming/rule/name_visitor.rs:366–386`

## code-ordering.rs.md

- [ ] **10S06** — false positive (to confirm) · kept (round 13): not a false positive, a payload type with a
  function-bearing `impl` has behaviour and its own file (`arch-package.rs.md:282–283`); the rule's test pins it
  - Doc (`code-ordering.rs.md:10–11`): "Then the file's type: its definition, its inherent `impl` block, then
    its trait `impl` blocks, then the data types only it uses, each with its own `impl` blocks."
  - Doc (`arch-domain.rs.md:100–101`): "A struct that only its variants carry sits in its file."
  - Gap: a trait `impl` or an inherent `impl` with functions on a payload type in its owner's file is reported
    as a second type with behaviour.
  - Rule: `OneTypeFamilyPerFile`, `family/rule/one_type_family_per_file.rs:113–147,225–229`

- [ ] **16S-S03** — missed · candidate (round 16; Sonnet): a const-only file module beside its type
  - Doc (`code-ordering.rs.md:59–60`): "A unit struct, an empty enum or a module beside a type that only holds that
    type's constants is forbidden."
  - Gap: only an inline `mod` is a constant holder; a file module of constants next to its owner's file passes.
  - Rule: `ConstantOutsideImpl`, `family/rule/constant_placement.rs:41–48,65–83`
- [ ] **16F-08** — false positive · candidate (round 16; Fable): a `static` in a `util` function file
  - Doc (`code-ordering.rs.md:7–8`): "then the file's `const` and `static` items and shared state, before any type or
    function"
  - Gap: a file `const` beside a function is exempt, a `static` of constant data beside the same function is
    reported.
  - Rule: `UtilShape`, `module/rule/util_shape.rs:28–42`

## di-manual-constructors.rs.md

- [ ] **12A09** — missed · kept (round 13): a foreign constructor in `new` is also the type's own state
  (`Notify::new()`, a channel)
  - Doc (`di-manual-constructors.rs.md:3–7`): "Never instantiate concrete dependencies inside a type's `new` … A
    dependency is a value with behaviour the type calls and a test might replace (a repository, a client, a
    clock)."
  - Gap: only implementations declared in the workspace are seen; `reqwest::Client::new()` inside `new` passes,
    while `RootModuleConstruction` does report calls into dependency crates.
  - Rule: `ConstructorDependency`, `module/rule/constructor_dependency.rs:71–113`

## di-manual.rs.md

- [ ] **9A05** — missed · kept (round 13): foreign shared state cannot be told from foreign constant data
  without a crate list
  - Doc (`di-manual.rs.md:143–145`): "no `static` holding a service or shared state (a `static` of `OnceLock`,
    `LazyLock`, `Mutex` or `RwLock`, `lazy_static!`). … A `static` holds only constant data."
  - Gap: a `static` of third-party shared state (`arc_swap::ArcSwapOption`) is accepted.
  - Rule: `GlobalState`, `module/rule/global_state_visitor.rs:66–98`
- [ ] **seat A row 1** — no rule · not mechanical, confirmed in round 13 (would need an invented list of DI crates)
  - Doc (`di-manual.rs.md:3`): "Manual constructor injection: no DI framework crate and no service locator."
  - Gap: no rule names a DI framework crate. The service-locator half is `GlobalState`.
- [ ] **seat A row 13** — no rule · not mechanical, confirmed in round 13 (needs a classifier of what a task does)
  - Doc (`di-manual.rs.md:141–142`): "A task that seeds or migrates a resource (`warmup`, `migration`) is built
    by the DI module of that resource and started by its `on_enable`."
- [ ] **11A08** — missed · kept (round 13): the installers of other logging crates are an open list
  - Doc (`di-manual.rs.md:145–146`): "The one global is the logging subscriber, which `main` installs once"
  - Gap: only the installers of `tracing`, `tracing_subscriber` and `log` are known; `env_logger::init()`,
    `fern::Dispatch::apply()` and `tracing_log::LogTracer::init()` outside `main` are accepted.
  - Rule: `GlobalState`, `module/rule/global_state_visitor.rs:56–64,165–186`
- [ ] **12A08 and 14A16, rest** — false positive · not taken (rounds 13 and 14): a `main` that waits for the
  shutdown signal through a `CancellationToken`, or through a method on a value (`signal.wait().await`)
  - Doc (`di-manual.rs.md:101–102`): "calls `on_enable` and waits for the shutdown signal"
  - Gap: such a `main` is rejected; a direct signal await, an awaited workspace free `async fn` and an awaited
    associated `async fn` holding the wait (one level) are accepted.
  - Rule: `MainShape`, `module/rule/main_shape_visitor.rs:158–189`

- [ ] **16A-H32** — false positive (minor) · candidate (round 16; Haiku): a forward through a local alias
  - Doc (`di-manual.rs.md:55–56`): "its methods call the same method of only the DI modules it owns, in order"
  - Gap: a child forward counts only through `self.<field>`; `let child = &self.child; child.on_enable();` is
    reported.
  - Rule: `DiModuleLifecycle`, `module/rule/lifecycle_body_visitor.rs:284–310`
- [ ] **16F-13** — false positive · candidate (round 16; Fable): a domain type named `…Module`
  - Doc (`di-manual.rs.md:5–8`): "A DI module is a Rust `struct` (`InvoiceModule`) that holds its services as
    fields, constructs their concrete instances in `new`, and receives other DI modules as parameters of `new`."
  - Gap: every struct whose name ends in `Module` outside `di` is reported, whatever it holds.
  - Rule: `DiPlacement`, `module/rule/di_placement.rs:105–117`

- [ ] **17A-S02** — missed · candidate (round 17; Sonnet): a subscriber built through a local
  - Doc (`di-manual.rs.md:145–146`): "The one global is the logging subscriber, which `main` installs once"
  - Gap: only an inline builder chain is recognised; `let builder = tracing_subscriber::fmt(); builder.init();`
    outside `main` passes.
  - Rule: `GlobalState`, `module/rule/global_state_visitor.rs:134–143`
  - Near: **11A08** — other logging crates.
- [ ] **17A-S03** — false positive (to confirm) · candidate (round 17; Sonnet): the logger in a tooling `main`
  - Doc (`di-manual.rs.md:145–146`): "The one global is the logging subscriber, which `main` installs once"
  - Gap: only `main` of an `entry-point` crate may install it; an `xtask` `main` is reported (a test pins it; no
    decision).
  - Rule: `GlobalState`, `module/rule/global_state_visitor.rs:202–207`
- [ ] **17A-H31** — missed (to confirm) · candidate (round 17; Haiku): a service behind `&dyn Trait` in a `static`
  - Doc (`di-manual.rs.md:143–146`): "no `static` holding a service or shared state … A `static` holds only constant
    data."
  - Gap: the service arm needs the resolved item to own an implementation, which a trait never does;
    `static DEFAULT_CLOCK: &dyn Clock = &SystemClock;` passes.
  - Rule: `GlobalState`, `module/rule/global_state_visitor.rs:178–185`
  - Near: **9A05** — third-party shared state.

- [ ] **19A-H03** — missed (weak) · candidate (round 19; Haiku): a restricted DI module visibility
  - Doc (`di-manual.rs.md:136`): "A DI module is `pub(crate)` unless another crate constructs it."
  - Gap: only `pub` items are read; a `pub(super)` or `pub(in …)` DI module passes.
  - Rule: `DiModuleVisibility`, `module/rule/di_module_visibility.rs:20–24`
- [ ] **19A-H04** — missed · candidate (round 19; Haiku): an early exit after `block_on`
  - Doc (`di-manual.rs.md:100–103`): "`block_on` returns `RootModule`, and `main` calls `on_disable` after it"
  - Gap: an early exit is recorded only before the shutdown wait; a `return` or `?` between `block_on` and
    `on_disable` passes.
  - Rule: `MainShape`, `module/rule/main_shape_visitor.rs:251–267`
- [ ] **19A-H05** — missed · candidate (round 19; Haiku): a service built by a method call in `RootModule`
  - Doc (`di-manual.rs.md:97–99`): "`RootModule` … constructs no service itself."
  - Gap: no method-call visit; `self.settings.build_pool()` in `RootModule` passes.
  - Rule: `RootModuleConstruction`, `module/rule/root_construction_visitor.rs:134–166`

## error-handling.rs.md

- [ ] **seat A row 39** — no rule · not mechanical, confirmed in round 13 (needs a classifier; clippy `panic` is not among the
  required lints)
  - Doc (`error-handling.rs.md:12`): "Never panic for an expected business or validation failure: a panic is
    reserved for bugs."
- [ ] **14A03, rest** — missed · not taken (round 14): a shadowing `let _name` written with a type annotation
  or a destructuring pattern
  - Doc (`error-handling.rs.md:155–156`): "never discard a `Result`: no call whose `Result` is left unused, and
    no `let _ =` on it, with one exception."
  - Gap: `let _x = call(); let _x: T = other(); use(_x);` is accepted: such a later binding still counts as a
    read of the first.
  - Rule: `ResultConsumption`, `runtime/rule/typed_runtime_visitor.rs:1068–1111`
- [ ] **14A04 and 15A01, rest** — missed · not taken (rounds 14 and 15): a `Result` the visitor cannot type
  - Doc (`error-handling.rs.md:155–156`): "never discard a `Result`: no call whose `Result` is left unused, and
    no `let _ =` on it, with one exception."
  - Gap: `_ = …;` and an unread `let _name = …;` are not reported when the call is a method on a receiver typed
    only by inference, on a generic or `dyn Trait` receiver, a `Self::…` path, or a foreign method or associated
    function (`_ = file.flush();`, `_ = stream.write(buf);`). A workspace call, the listed `std::fs` and
    `tokio::fs` functions and an awaited `JoinHandle` are reported, and `.ok();` on any `Result` goes to
    `clippy::unused_result_ok` (F38).
  - Rule: `ResultConsumption`, `runtime/rule/typed_runtime_visitor.rs:1017–1148`
- [ ] **14A05, rest** — missed · not taken (round 14): a caught closure that calls another local closure
  - Doc (`error-handling.rs.md:101–102`): "Your own logic returns `Err` and is never wrapped in
    `std::panic::catch_unwind`"
  - Gap: `let step = || own_fn(); let work = || step(); catch_unwind(work)` is accepted; the closure a caught
    closure calls is not followed.
  - Rule: `CatchUnwind`, `runtime/rule/task_visitor.rs:441–505`
- [ ] **14A12, rest** — missed · not taken (round 14): two `?` steps in the tail of a closure body
  - Doc (`error-handling.rs.md:110–111`): "chain steps with `?`, one step per `let`, and the happy path stays
    flat"
  - Gap: `|raw| save(parse(raw)?)` composes two `?` steps in a closure's tail and is accepted.
  - Rule: `ResultConsumption`, `runtime/rule/result_consumption.rs:56–114`

- [ ] **16A-H02** — missed · candidate (round 16; Haiku): an error enum not named `*Error`
  - Doc (`error-handling.rs.md:74–75`): "Any other error enum is its own file in the `model` of the level that uses
    it, even one that never leaves one implementation, never a private enum inside the file that returns it."
  - Gap: error types are selected by name, `Error` derive or `Error` impl; `enum ParseFailure` used as `E` in
    `Result<T, E>` is seen by neither rule.
  - Rule: `ErrorPlacement`, `module/rule/error_placement.rs:14–31`; `ErrorFamilyShape`, `declaration/rule/error_family_shape.rs:130`
- [ ] **16A-H14** — missed · candidate (round 16; Haiku, Fable): a foreign error type not named `*Error`
  - Doc (`error-handling.rs.md:27–31`): "so the family names no foreign error type and stays `Send + Sync`"
  - Gap: a field without `#[source]` or `#[from]` is foreign only when its type ends in `Error`;
    `Timeout(tokio::time::error::Elapsed)` passes.
  - Rule: `ErrorFamilyShape`, `declaration/rule/error_family_shape.rs:136`
- [ ] **16F-14** — false positive (to confirm) · candidate (round 16; Fable): `#[source]` on the project's own
  family
  - Doc (`error-handling.rs.md:28–30`): "A cause from code you do not own is the `#[source]` field `source: Box<dyn
    Error + Send + Sync>`"
  - Gap: every `#[source]` field that is not the box is reported, the project's own family included; the document
    prescribes `#[from]` for own families and is silent on `#[source]`.
  - Rule: `ErrorFamilyShape`, `declaration/rule/error_family_shape.rs:134–137`
- [ ] **16A-H30** — missed · candidate (round 16; Haiku): log, then return the error wrapped
  - Doc (`error-handling.rs.md:155`): "Log a failure once, at the call site that has enough context. Do not log and
    return it with `?`"
  - Gap: a log is linked to `Err(x)` only when `x` is a bare name; `return Err(Wrapper { source: error })`,
    `Err(error.into())` and `Err(Wrapper::from(error))` after the log pass.
  - Rule: `ErrorLogging`, `runtime/rule/error_logging.rs:377–396`
- [ ] **16A-H31** — missed · candidate (round 16; Haiku): log in `if let`, then `?` on the result
  - Doc (`error-handling.rs.md:155`): "Do not log and return it with `?`"
  - Gap: names bound in an `if let` condition leave `logged` after the branch;
    `if let Err(error) = &result { error!(…) } result?;` passes.
  - Rule: `ErrorLogging`, `runtime/rule/error_logging.rs:494–513`

- [ ] **17A-S06** — missed (to confirm) · candidate (round 17; Sonnet): a discarded aliased `Result`
  - Doc (`error-handling.rs.md:155–156`): "never discard a `Result`"
  - Gap: a workspace function's return is compared by its outer path; `_ = write_report();` returning
    `std::io::Result<()>` passes.
  - Rule: `ResultConsumption`, `runtime/rule/typed_runtime_visitor.rs:19,1040–1046`
  - Near: **14A04 and 15A01, rest** — untyped receivers.
- [ ] **17A-H02** — missed (weak) · candidate (round 17; Haiku): a `!Send` field in an error family
  - Doc (`error-handling.rs.md:29–30`): "so the family names no foreign error type and stays `Send + Sync`"
  - Gap: no `Send`/`Sync` check; a field `Rc<str>` passes.
  - Rule: `ErrorFamilyShape`, `declaration/rule/error_family_shape.rs:130–140`
- [ ] **17A-H04** — missed · candidate (round 17; Haiku): `Self::` calls inside `catch_unwind`
  - Doc (`error-handling.rs.md:101–102`): "Your own logic returns `Err` and is never wrapped in
    `std::panic::catch_unwind`"
  - Gap: `Self` resolves as a builtin, so `catch_unwind(|| Self::parse(raw))` is not own logic.
  - Rule: `CatchUnwind`, `runtime/rule/task_visitor.rs:481–491`
  - Near: **14A05, rest** — a local closure chain.

- [ ] **19A-H07** — missed (weak) · candidate (round 19; Haiku): `?` steps inside a macro argument
  - Doc (`error-handling.rs.md:110–111`): "chain steps with `?`, one step per `let`"
  - Gap: the step counter does not enter macro tokens; `let line = format!("{}{}", a()?, b()?);` counts 0.
  - Rule: `ResultConsumption`, `runtime/rule/fallible_step_visitor.rs:43–52`
  - Near: **14A12, rest** — a closure tail.

- [ ] **20A-S01** — no rule (to confirm) · candidate (round 20; Sonnet): a foreign error type outside a contract
  - Doc (`error-handling.rs.md:85–88`): "so inner layers never see a foreign error type"
  - Gap: only `api` trait methods are checked; a `usecase`, `util` or `model` function returning
    `Result<_, std::io::Error>` passes. Host-required signatures would need an exemption.
  - Rule: none; nearest `ContractShape`, `declaration/util/contract_method_shape.rs:184–235`
- [ ] **20A-S03** — missed (weak) · candidate (round 20; Sonnet): a route handler written as a method
  - Doc (`error-handling.rs.md:13–16`): "returned only from that framework's entry point (`route`)"
  - Gap: the route error check reads free `fn` items only.
  - Rule: `ErrorPlacement`, `module/rule/error_placement.rs:106–113`

## function-reference.rs.md

- [ ] **14S07, rest** — missed · not taken (round 14): the two-step closure after a split of a `&str` local, and
  a closure with a pattern parameter
  - Doc (`function-reference.rs.md:28–29`): "When the closure first adapts its parameter with a method and then
    calls one function, move the adaptation into the chain and use the path"
  - Gap: the adapt-then-call closure is reported after a split of a `&str` parameter or literal; on a `&str`
    local, or with a pattern parameter, it is accepted.
  - Rule: `TrivialClosurePath`, `expression/rule/adapter_chain_visitor.rs:50–69,159–208`

- [ ] **17S-S05** — missed · candidate (round 17; Sonnet): adapt-then-call over a non-string iterator
  - Doc (`function-reference.rs.md:28–29`): "When the closure first adapts its parameter with a method and then calls
    one function"
  - Gap: a `&str`-lines receiver is required; `items.iter().map(|item| to_row(item.clone()))` passes.
  - Rule: `TrivialClosurePath`, `expression/rule/adapter_chain_visitor.rs:166–168`

## imports.rs.md

- [ ] **11T02** — no rule · kept (round 13): the order is rustfmt's; its carrier is `cargo fmt --check` in the
  consumer's CI
  - Doc (`imports.rs.md:7–8`): "in the order `cargo fmt` gives: rustfmt sorts a block of consecutive `use` lines"
  - Gap: nothing compares the order of the `use` paths; only gaps, comments and position are checked, and no
    required configuration makes a consumer run `cargo fmt --check`.
  - Rule: `ImportOrdering`, `import/rule/import_order_visitor.rs:34–53`
- See also **7T05** under `notes.rs.md` (`imports.rs.md:3`: "Never use wildcard imports").

## no-callbacks.rs.md

- [ ] **7A06** — no rule · closed by decision (F25; the document was then edited to say so), confirmed in
  round 13
  - Doc (`no-callbacks.rs.md:15–17`): "It does not flag a closure inside a container field
    (`Vec<Box<dyn Fn…>>`, `HashMap<…, Box<dyn Fn…>>`) …, so that field stays with the reviewer."
  - Gap: a closure inside a container field is not reported. The document now agrees with the code.
- [ ] **seat A row 57** — no rule · not mechanical, confirmed in round 13 (the choice of a capacity is semantic)
  - Doc (`no-callbacks.rs.md:53–56`): "a `broadcast` channel for an event several subscribers read. Its
    capacity is a required argument … an `mpsc` channel with an explicit capacity for events one consumer must
    not lose."
- [ ] **seat A row 60** — no rule · not mechanical, confirmed in round 13 (the Bad form is a `bool` field in a state struct)
  - Doc (`no-callbacks.rs.md:105–106`): "A one-off event from a state holder to the one consumer that acts on it
    … is an `mpsc` channel, never a flag in the state that the consumer has to clear, and never a `broadcast`"
- [ ] **seat A row 67** — no rule · not mechanical, confirmed in round 13 (what is "work that ends" needs a classifier)
  - Doc (`no-callbacks.rs.md:282–283`): "Blocking work that ends (a file read, a blocking client call, a heavy
    computation) runs in `tokio::task::spawn_blocking` and is awaited."
- [ ] **11A04, rest** — missed · not taken (round 13): `receiver.recv().await?` inside a `loop`
  - Doc (`no-callbacks.rs.md:59–60`): "A loop over a `broadcast::Receiver` matches `Lagged` and goes on.
    `while let Ok(event) = receiver.recv().await` ends at the first lag"
  - Gap: the `?` form ends the loop at the first lag and is not reported; `while let`, `match`, `let … else`
    and `if let Ok(…) … else { break }` are.
  - Rule: `BroadcastLag`, `runtime/rule/typed_runtime_visitor.rs:1078–1091,1491–1499`
- [ ] **14A13, rest** — missed · not taken (round 14): a blocking receive in `fn drop`
  - Doc (`no-callbacks.rs.md:261–263`): "`Drop` is synchronous and short … a destructor that may block needs a
    separate teardown method"
  - Gap: a blocking channel receive (`recv()` of `std::sync::mpsc`, `blocking_recv`) in `fn drop` is accepted;
    `block_on` and `std::thread::sleep` there are reported, and the join of an owned thread stays allowed
    (`no-callbacks.rs.md:275`).
  - Rule: `BlockingLockInSync`, `runtime/rule/typed_runtime_visitor.rs:1432–1434,1776–1786`

- [ ] **16A-S06** — missed · candidate (round 16; Sonnet): a registration guard dropped at once by a statement
  - Doc (`no-callbacks.rs.md:142–144`): "A registration guard bound to a named variable (`_registration`; `let _ =`
    would drop it at once)"
  - Gap: only `let _ = …` is recognised; the bare statement `Guard::new(..);` and `_ = Guard::new(..);` pass.
  - Rule: `CallbackBridgeShape`, `runtime/rule/task_visitor.rs:1705–1740`
- [ ] **16A-S09** — missed · candidate (round 16; Sonnet, Haiku): an enum implementing a `*Listener` trait
  - Doc (`no-callbacks.rs.md:11–12`): "a type in the `event` kind that implements the platform's own listener trait
    declares no callback"
  - Gap: only structs are read; an enum implementing a host `*Listener` trait outside `event` passes.
  - Rule: `ListenerName`, `module/rule/listener_name.rs:20–22`
- [ ] **16A-S12** — missed · candidate (round 16; Sonnet): a contract returning a `std` or `futures` receiver
  - Doc (`no-callbacks.rs.md:134–137`): "The contract in `api` declares only `async fn`s, `watch::Receiver`s and
    streams"
  - Gap: only tokio's `broadcast`/`mpsc` receivers and a `oneshot::Receiver` are rejected; `std::sync::mpsc::Receiver`
    and `futures::channel::mpsc::Receiver` pass, while `std::sync::mpsc::Sender` as a parameter is reported.
  - Rule: `ContractShape`, `declaration/util/contract_method_shape.rs:136–153`
- [ ] **16A-H10** — missed · candidate (round 16; Haiku): blocking `std::net` and `std::process` calls in an
  `async fn`
  - Doc (`no-callbacks.rs.md:286–287`): "An `async fn` never blocks its worker thread"
  - Gap: only `std::thread::sleep`, `std::fs`, `std::sync::mpsc` and `std::fs::File` methods are blocking;
    `TcpStream` I/O and `Command::output`/`status` pass, though `is_blocking_io_method` knows them.
  - Rule: `BlockingCallInAsync`, `runtime/rule/task_visitor.rs:1371–1377`; `typed_runtime_visitor.rs:565–578`
- [ ] **16A-H15** — missed · candidate (round 16; Haiku): a free `block_on` in `Drop` of a technology module
  - Doc (`no-callbacks.rs.md:261–263`): "a destructor that may block needs a separate teardown method"
  - Gap: only a `block_on` method on a `Handle`/`Runtime` is matched in `drop`; `futures::executor::block_on(..)`
    passes, and `BlockOnOutsideBoundary` exempts technology modules.
  - Rule: `BridgeCleanup`, `runtime/rule/task_visitor.rs:336–347`
  - Near: **14A13, rest** — a blocking channel receive in `drop`.

- [ ] **17A-S05** — missed (weak) · candidate (round 17; Sonnet): a thread handle never joined
  - Doc (`no-callbacks.rs.md:275`): "the owner joins the thread when it stops"
  - Gap: only discard forms are read; `let worker = std::thread::spawn(..);` never joined passes.
  - Rule: `ThreadPrimitive`, `runtime/rule/task_visitor.rs:1361–1365`

- [ ] **20A-S05** — false positive (to confirm) · candidate (round 20; Sonnet): the prescribed send from a `match`
  arm
  - Doc (`no-callbacks.rs.md:140–142`): the bridge's `let _ = sender.send(outcome)` under the documented expectation
  - Gap: a `Some(sender)` bound in a `match` arm is untyped, so the discard is reported; the `let … else` spelling
    passes.
  - Rule: `ResultConsumption`, `runtime/rule/typed_runtime_visitor.rs:1704–1722,1950–1957`

## no-print.rs.md

- [ ] **16A-S01** — missed · candidate (round 16; Sonnet, Fable): `#[expect(clippy::disallowed_methods)]` is
  accepted on any item
  - Doc (`no-print.rs.md:29–32`): "`#[expect(clippy::disallowed_methods, …)]` when it writes through
    `std::io::stdout()`"; (`time-api.rs.md:76–78`): "No other clock read is called"; (`async.rs.md:22–25`): "`send`
    is never called on a state holder"
  - Gap: the expectation is prescribed for every role and item (only `print_stdout`/`print_stderr` are gated to
    entry-point and tooling crates), so every `disallowed-methods` entry without a rule of its own escapes under
    it: `std::io::stdout`/`stderr` outside a CLI, the time-crate clock reads and `SystemTime::elapsed`,
    `watch::Sender::send`. `SystemClockOutsideDi` covers `SystemTime::now` only. Merges 16A-S02, 16A-S07, 16F-12.
  - Rule: `ExpectScope`, `suppression/rule/lint_expectation_visitor.rs:24–34`
- [ ] **16A-S11** — missed · candidate (round 16; Sonnet): an error from `Result::err()`
  - Doc (`no-print.rs.md:6–7`): "A caught error is passed to the logger as a field of the event"
  - Gap: an error binding is recognised only from an `Err(..)` pattern; `if let Some(error) = result.err()` and a
    message formatting it pass.
  - Rule: `ErrorLogging`, `runtime/rule/error_logging.rs:219–241`

## no-tuples.rs.md

- [ ] **16S-S02** — missed · candidate (round 16; Sonnet): a tuple from a method chain bound to one name
  - Doc (`no-tuples.rs.md:6–9`): "it is never bound to one name (`let parts = text.split_at(index);`)"
  - Gap: a std receiver is recognised only as a literal or a typed single-name path; `text.trim().split_at(3)`
    and `text.trim().char_indices()` pass.
  - Rule: `AnonymousTuple`, `tuple/rule/tuple_binding_visitor.rs:130–153,248–275`

- [ ] **17S-S06** — missed · candidate (round 17; Sonnet): a tuple bound to one name in a pattern
  - Doc (`no-tuples.rs.md:6–9`): "it is never bound to one name"
  - Gap: only `let name = <tuple>` is checked; `if let Some(pair) = text.split_once('=')`, a `match` arm and
    `while let` pass.
  - Rule: `AnonymousTuple`, `tuple/rule/tuple_binding_visitor.rs:624–630`
- [ ] **17S-S07** — missed · candidate (round 17; Sonnet): `split_first` and its siblings
  - Doc (`no-tuples.rs.md:6–9`): "it is never bound to one name"
  - Gap: a closed method list; `let parts = values.split_first().expect("non-empty");` passes.
  - Rule: `AnonymousTuple`, `tuple/rule/tuple_binding_visitor.rs:352–364,411–419`
- [ ] **17S-H31** — false positive (document conflict) · candidate (round 17; Haiku): a tuple associated type
  - Doc (`no-tuples.rs.md:17–19`): "A method of an `impl Trait for` block, whose signature the trait gives"
  - Gap: `type Item = (u8, u8);` in an `impl Iterator for …` is reported; the exemption names methods only.
  - Rule: `AnonymousTuple`, `tuple/rule/tuple_type_visitor.rs:54–67,145–153`
- [ ] **18S-S02** — missed · candidate (round 18; Sonnet): a tuple bound to one name in a trait's default method
  - Doc (`no-tuples.rs.md:6–9`): "it is never bound to one name"
  - Gap: parameter types are registered for free functions and `impl` methods only; `let parts = text.split_at(1);`
    in a default trait method passes.
  - Rule: `AnonymousTuple`, `tuple/rule/tuple_binding_visitor.rs:588–602`

## notes.rs.md

- [ ] **7T05 (10T07)** — missed · closed by decision (F29: the rule's committed fixtures pin the root file only),
  confirmed in round 13
  - Doc (`notes.rs.md:11`): "`Unanalysable`: a limit of the linter, not of the code (a macro it cannot see
    through)"
  - Doc (`imports.rs.md:3`): "Never use wildcard imports"
  - Gap: an item-position macro call in a file module is not an `Unanalysable` site, and the code inside it (a
    glob import in `cfg_if!`, an `unsafe`) is read by no rule.
  - Rule: `Unanalysable`
- [ ] **11T05** — no rule · kept (round 13): "unrelated" is a judgement; equal reasons may be rightly shared
  - Doc (`notes.rs.md:21`): "One reason covers one item: there is no baseline and no reason shared by unrelated
    items."
  - Gap: only a missing or blank reason is rejected; the same reason string on any number of `expect`
    attributes or manifest subjects passes.
  - Rule: none; nearest `suppression/` (`expectation_parser.rs:57–68`, `expectation_validate.rs:54–101`)

- [ ] **17T-S01** — missed · candidate (round 17; Sonnet): an old dated stable toolchain
  - Doc (`notes.rs.md:4–5`): "the pin is a stable release at or above 1.93"
  - Gap: any `stable-YYYY-MM-DD` passes before the floor test; `stable-2024-03-01` (1.76) is accepted.
  - Rule: `LintConfig`, `config/rule/lint_config.rs:454–477`
- [ ] **17T-S02** — missed · candidate (round 17; Sonnet): a blank `reason`
  - Doc (`notes.rs.md:7`): "`#[expect(<lint>, reason = "<why>")]`"
  - Gap: the reason value is never read; `reason = ""` passes.
  - Rule: `ExpectScope`, `suppression/rule/lint_expectation_visitor.rs:36–82`
  - Near: **11T05** — a reason shared by unrelated items.
- [ ] **17T-H32** — missed (weak) · candidate (round 17; Haiku): a group allowed at a required lint's priority
  - Doc (`notes.rs.md:6–7`): "a finding is fixed, not silenced"
  - Gap: a lowering group is reported only above the lowest required priority (`>`), while the carrier branch
    uses `>=`.
  - Rule: `LintConfig`, `config/rule/lint_config.rs:113`

- [ ] **19T-S01** — false positive (weak, to confirm) · candidate (round 19; Sonnet): a channel with a host triple
  - Doc (`notes.rs.md:4–5`): "the pin is a stable release at or above 1.93"
  - Gap: `1.93.0-x86_64-unknown-linux-gnu` is rejected; only a version or `stable[-date]` parses.
  - Rule: `LintConfig`, `config/rule/lint_config.rs:465–477`

## region.md

- [ ] **16S-S01** — missed (implemented case 9S01, rest) · candidate (round 16; Sonnet, Haiku): one-sided and
  non-ASCII section headers
  - Doc (`region.md:19`): "Do not add decorative comments, separator comments, or "section header" comments."
  - Gap: a one-sided run counts only for `=`, `-`, `*`; `// ### Helpers` and `// ─── Helpers ───` pass.
  - Rule: `ForbiddenComment`, `comment/util/comment_shape.rs:2–3,41–51`
- [ ] **16S-H40** — false positive (minor) · candidate (round 16; Haiku): prose that starts with "region"
  - Doc (`region.md:3`): "Region markers and artificial section delimiters are forbidden."
  - Gap: any non-alphanumeric character after `region` makes a marker; `// region, country and city …` is reported.
  - Rule: `ForbiddenComment`, `comment/util/comment_shape.rs:6–17`
- [ ] **16F-11** — missed · candidate (round 16; Fable): other region-marker forms
  - Doc (`region.md:3`): "Region markers and artificial section delimiters are forbidden."
  - Gap: only `region`/`endregion` are markers; `// <editor-fold>` and `// MARK: -` pass.
  - Rule: `ForbiddenComment`, `comment/util/comment_shape.rs:6–17`

- [ ] **17S-S03** — false positive (minor) · candidate (round 17; Sonnet): Markdown bold-italic at line start
  - Doc (`region.md:19`): "Do not add decorative comments, separator comments, or "section header" comments."
  - Gap: three leading `*` make a one-sided header; `/// ***Never*** hold the guard …` is reported.
  - Rule: `ForbiddenComment`, `comment/util/comment_shape.rs:48–50`

## rustdoc.rs.md

- [ ] **9S02** — missed · kept (round 13): a getter's doc may carry units or ranges; the word-for-word repeat
  is already reported
  - Doc (`rustdoc.rs.md:65`): "Comments on trivial getters, structs with self-explanatory fields, or trait impls
    that don't change the trait's contract."
  - Gap: a doc comment on a trivial getter of a published library is not reported; no rule covers the getter
    part.
  - Rule: `DocCommentScope`, `comment/rule/doc_scope_visitor.rs:178–197`
- [ ] **11S01** — no rule · kept (round 13): the exception "a reason that points outside the code" is the
  reviewer's call in the document
  - Doc (`rustdoc.rs.md:9`): "In a plugin, an application or a service the code is understandable without doc
    comments, and none is written."
  - Gap: a plain prose doc comment on an item of an unpublished crate is accepted whatever it says; a non-`fn`
    item is never reported, and a `fn` only for repeating its name or a trait's doc.
  - Rule: `DocCommentScope`, `comment/rule/doc_scope_visitor.rs:47–78,151–227`
- [ ] **14S04** — not mechanical · kept (round 14): a parameter line that paraphrases the name, and a
  signature-restating summary, are not reported; telling a paraphrase needs a list of synonyms
  - Doc (`rustdoc.rs.md:62–63`): "A parameter description that restates the name: "`user_id` — the identifier
    of the user"."
  - Gap: only a word-for-word restatement is reported; a paraphrase (`identifier of the user`) and a summary
    that restates the signature need a synonym classifier.
  - Rule: `RustdocFiller`, `comment/rule/rustdoc_parameter_visitor.rs:46–50`
  - Near: **9S02** — that item is the getter and struct case; this is paraphrase and summary restatement.

- [ ] **16S-H23** — missed (weak) · candidate (round 16; Haiku): filler after another item word
  - Doc (`rustdoc.rs.md:62`): "Filler and signature restatement: "This function is responsible for...""
  - Gap: the filler predicates are checked after six item words; `//! This module is responsible for …` passes.
  - Rule: `RustdocFiller`, `comment/rule/rustdoc_filler.rs:19–23,36`
- [ ] **16S-H42** — missed · candidate (round 16; Haiku): a hyphenated HTML tag
  - Doc (`rustdoc.rs.md:67`): "HTML tags (`<p>`, `<code>`, `<br>`). Use Markdown and intra-doc links."
  - Gap: a tag name is ASCII alphanumerics only; `<my-widget>` passes.
  - Rule: `RustdocHtmlTag`, `comment/rule/rustdoc_html_tag.rs:22–30`

- [ ] **17S-S01** — missed · candidate (round 17; Sonnet, Haiku): parameter restatements in other line shapes
  - Doc (`rustdoc.rs.md:63`): "A parameter description that restates the name"
  - Gap: only ``- `name` — words`` (or `-`) is read; a `* ` bullet, a `:` or `–` separator, an unquoted name, or
    one bullet of several passes.
  - Rule: `RustdocFiller`, `comment/rule/rustdoc_parameter_visitor.rs:27–51,100–118`
  - Near: **14S04** — paraphrase.
- [ ] **17S-H52** — missed (weak) · candidate (round 17; Haiku): indexing a map or deque parameter
  - Doc (`rustdoc.rs.md:5`): "`# Panics` is documented when a function of a published library panics on input or
    state its caller controls"
  - Gap: `[]` on a `HashMap`, `BTreeMap` or `VecDeque` parameter is not a proven panic.
  - Rule: `DocCommentScope`, `comment/rule/panic_doc_visitor.rs:156–172,255–258`
- [ ] **17S-H55** — not mechanical · candidate (round 17; Haiku): a divisor taken from a local
  - Doc (`rustdoc.rs.md:5`): "panics on input or state its caller controls"
  - Gap: only a parameter divisor is read; whether a local carries caller input needs data flow.
  - Rule: `DocCommentScope`, `comment/rule/panic_doc_visitor.rs`

## struct-default-values.rs.md

- [ ] **14S11, rest** — missed · not taken (round 14): the handlers of a `select!` for the visitors that
  resolve names and for the other visitors that read macro arguments as a comma list
  - Doc (`struct-default-values.rs.md:3–4`): "Do not give a struct or an enum default values"
  - Doc (`no-tuples.rs.md:8–9`): "it is never bound to one name"
  - Gap: a workspace item named only inside a `select!` handler is no user of it (a branch pattern binds names
    the resolver does not know), and the visitors other than those 14S11 named skip the handler bodies.
  - Rule: `StructDefaultValue`, `default/rule/struct_update_visitor.rs:89–93`; the name-resolving visitors
- [ ] **18S-S01** — false positive (to confirm) · candidate (round 18; Sonnet): struct update on an exempt type
  - Doc (`struct-default-values.rs.md:42–44`): "Allowed exceptions: * A type that derives `serde::Deserialize`, where
    defaults are required for serialization"
  - Gap: `..Default::default()` is reported on any own struct; the derive and `impl` branches apply the exception,
    the update branch does not.
  - Rule: `StructDefaultValue`, `default/rule/struct_update_visitor.rs:135–149`

## testing-no-useless.rs.md

- [ ] **14S03** — not mechanical · kept (round 14): the Bad example of the document is reported by nothing;
  the sentence needs a judgement of what a test verifies, and the one syntactic form (a field asserted against
  the literal it was built with) would be a new rule for a textbook case, which is the owner's call
  - Doc (`testing-no-useless.rs.md:3,5–17`): "Do not write tests that only verify language/library behavior or
    trivial data storage."
  - Gap: the document's own Bad example (a field asserted against the literal it was just built with) has no
    carrier; the gate records it as review. Only a narrow literal-then-field form is syntactic.
  - Rule: none; nearest `ModelRuleTest`

## testing.rs.md

- [ ] **11S04** — no rule · kept (round 13): which `multi_thread` test is a race is not in the attribute
  - Doc (`testing.rs.md:31–32`): "A race needs worker threads:
    `#[tokio::test(flavor = "multi_thread", worker_threads = 2)]`."
  - Gap: nothing reads `worker_threads`; `#[tokio::test(flavor = "multi_thread")]` without it is accepted.
  - Rule: none; nearest `TestTimePaused`, `runtime/rule/test_runtime_visitor.rs:464–487`
- [ ] **12S04** — no rule · kept (round 13): "Prefer fakes over mocks" is a preference; the documents list no
  mocking crate
  - Doc (`testing.rs.md:11`): "Prefer fakes over mocks for repositories and gateways when behavior matters."
  - Doc (`testing.rs.md:183–184`): "**Frameworks**: the built-in `#[test]` harness, `#[tokio::test]` with tokio's
    `test-util` feature in `[dev-dependencies]`."
  - Gap: a mocking crate in `[dev-dependencies]` or `#[automock]` on a contract is reported by nothing.
  - Rule: none; nearest `FakePlacement`, `LintConfig`
- [ ] **15S02** — no rule · not mechanical (round 15): 100% line and branch coverage of new project behaviour is a
  measured quantity and a CI step, which no source checker reads
  - Doc (`testing.rs.md:20–23`): "New code that implements project behaviour MUST have 100% line coverage and
    100% branch coverage through meaningful behaviour tests."
  - Doc (`testing-no-useless.rs.md:32`): "The testing rule demands 100% line coverage and 100% branch coverage of
    new code that implements project behaviour."
  - Gap: nothing in the linter or in the required configuration reads coverage; the documents name no fixed file
    a rule could read. `decisions.md` F16 drops the percentage for the linter's own CI only.
  - Rule: none
  - Near: `notes.rs.md:5` under "Not in this list" (a CI step of the consumer, not counted by round 12).

- [ ] **16S-S05** — not mechanical · candidate (round 16; Sonnet): the Bad test focus
  - Doc (`testing.rs.md:80–87`): "The second test verifies an implementation detail."
  - Gap: a test named for a call count on a fake is reported by nothing; telling a behaviour name from an
    implementation-detail name needs a classifier.
  - Rule: none; nearest `TestFunctionName`
- [ ] **16S-S09** — missed · candidate (round 16; Sonnet, Haiku): `ModelRuleTest` purity is too narrow
  - Doc (`testing.rs.md:5`): "a model rule is called directly in a plain `#[test]`, with no runtime"
  - Gap: a util rule with any method call is impure, and a test with a cast, closure, array, tuple, index, `?` or a
    non-identifier `let` pattern (`let _ = …`, round 20) is impure; a `#[tokio::test]` without `.await` over such a rule passes. The README states "proven pure" as the
    bound; no decision does.
  - Rule: `ModelRuleTest`, `runtime/rule/test_runtime_visitor.rs:57–89,303–343`
- [ ] **16S-H45** — missed (weak) · candidate (round 16; Haiku): an executor the test builds
  - Doc (`testing.rs.md:35–37`): "never a runtime the test builds or blocks on"
  - Gap: only a free `block_on` and runtime construction are read; `futures::executor::LocalPool::new().run_until(..)`
    passes.
  - Rule: `BlockOnInTest`, `runtime/rule/runtime_visitor.rs:75–82`

- [ ] **17S-H35** — false positive · candidate (round 17; Haiku): a domain type named `Fake*`
  - Doc (`testing.rs.md:159–161`): "A same-crate fake lives in `fake/` inside the concept of the trait it replaces"
  - Gap: any type named `Fake*` is a fake; `FakeNewsReport` in `model` is reported.
  - Rule: `FakePlacement`, `module/rule/fake_placement.rs:24–26,73`
- [ ] **17S-H37** — missed (weak) · candidate (round 17; Haiku): `LocalSet::block_on` in a test
  - Doc (`testing.rs.md:35–37`): "never a runtime the test builds or blocks on"
  - Gap: the `LocalSet` receiver counts only for `BlockOnOutsideBoundary`.
  - Rule: `BlockOnInTest`, `runtime/rule/typed_runtime_visitor.rs:514–520,1994–2002`
  - Near: **16S-H45** — `LocalPool`.

- [ ] **19S-H02** — missed (weak) · candidate (round 19; Haiku): waiting on time through a helper
  - Doc (`testing.rs.md:37–38`): "A test that waits on time uses `#[tokio::test(start_paused = true)]`"
  - Gap: only the test's own body is read; a workspace helper that sleeps passes.
  - Rule: `TestTimePaused`, `runtime/rule/test_runtime_visitor.rs:531–579`
- [ ] **19S-H03** — false positive (to confirm) · candidate (round 19; Haiku): a timeout guard that never waits
  - Doc (`testing.rs.md:37–38`): "A test that waits on time"
  - Gap: `tokio::time::timeout` in an unpaused test is reported even around a future that completes at once.
  - Rule: `TestTimePaused`, `runtime/rule/test_runtime_visitor.rs:556–577`

## time-api.rs.md

- [ ] **11A10** — missed · kept (round 13): the extra unit suffixes come from no document or primary source
  (`_MIN` is also "minimum")
  - Doc (`time-api.rs.md:43–46`): "the constant stays a `Duration`
    (`const CARD_IN: Duration = Duration::from_millis(400);`, never `const CARD_IN_MS: u64 = 400;`)"
  - Gap: the unit suffixes of an integer constant are `_MS`, `_MILLIS`, `_SECS`, `_SECONDS`, `_NANOS`,
    `_MICROS`, `_MINUTES`, `_HOURS`; `_DAYS`, `_SEC`, `_MIN`, `_MILLISECONDS`, `_NS`, `_US` are not reported.
  - Rule: `TimePrimitive`, `runtime/rule/time_primitive.rs:152–159,386–411`
- [ ] **14A11** — not mechanical · kept (round 14): a moment held as an integer or text under a name that
  carries no unit; the document gives the call to the reviewer, and a name list (`_at`, `_time`) would guess
  - Doc (`time-api.rs.md:3–4`): "Which values are moments or durations is the reviewer's call."
  - Gap: `created_at: i64` and `expires_at: String` (the first two lines of the Bad example) cannot be told from
    other integers without a name classifier.
  - Rule: none
  - Near: **14A10** — the third line of the same Bad example, `timeout_seconds: u32`, carries a unit suffix the
    rule reads on fields and constants.

- [ ] **16A-H12** — missed · candidate (round 16; Haiku): a unit-suffixed integer parameter
  - Doc (`time-api.rs.md:42–46`): "Only convert moments or durations to integers or strings at external boundaries"
  - Gap: unit suffixes are read on fields and constants only; `fn retry(timeout_seconds: u64)` in a non-boundary
    module passes. The README states "not a function parameter" as the bound; no decision does.
  - Rule: `TimePrimitive`, `runtime/rule/time_primitive.rs:501–512`

- [ ] **17A-S01** — missed · candidate (round 17; Sonnet): a unit suffix on a wrapped integer
  - Doc (`time-api.rs.md:42–46`): "Only convert moments or durations to integers or strings at external boundaries"
  - Gap: only a bare primitive or `String` is read; `timeout_seconds: Option<u64>` and `delays_ms: Vec<u64>` pass.
  - Rule: `TimePrimitive`, `runtime/rule/time_primitive.rs:143–163`
  - Near: **11A10** — the suffix list.
- [ ] **17A-H01** — missed (weak) · candidate (round 17; Haiku): a unit-suffixed local
  - Doc (`time-api.rs.md:3–4`): "do not represent moments in time or durations as raw primitive or text types"
  - Gap: locals are not read; `let timeout_ms: u64 = 500;` passes (the README scopes the check; no decision).
  - Rule: `TimePrimitive`, `runtime/rule/time_primitive.rs:429–442`

## Not in this list

- The cases that are in the code (implemented, or fixed by a document change): see the second paragraph.
- The 73 clause groups round 9 gave to the reviewer or to an agent's process (meaning of a name, ownership, what
  a context is): the allocation tables of `reaudit-2026-11-10/seat-{t,s,a}.md` list them.
- Cases where the code already follows the document: F65 (a comment between `use` lines is reported), 9S07
  (`it` as a fn-pointer parameter name is reported), 7A04 and F16 (the committed Bad examples pin the reported
  code as a violation).
- The Kotlin linter. Its gaps were last audited on 2026-10-30/31 in `.claude/handoff/lint-audit-matrix.md` and
  `lint-fix-plan.md`; those lists predate later fixes and document changes.
- `notes.rs.md:5` ("`cargo clippy --workspace --all-targets -- -D warnings` runs in CI"): a CI step of the
  consumer repository; the document names no fixed file a rule could read, so round 12 did not count it.
- The rounds did not converge (20, 35, 22, 19, 18, 29, 22 and 26 new cases in rounds 5–12, 35 in round 14, 8 in
  round 15: 6 by its audit, 2 while its cases were fixed, 64 in round 16 (one withdrawn in round 17), 51 in round 17, 4 in round 18, 15 in round 19, 6 in round 20, where the user stopped), so more cases exist than this file names. Round 15
  was read by one reader, from the documents and about a dozen rule sources, not from all 138 rules.
