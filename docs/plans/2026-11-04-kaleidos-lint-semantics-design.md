# Kaleidos lint semantics

The user requested two further design sessions with Sol and Astra and authorized implementing the best result. Both sessions reviewed all six findings and the underlying bound-reference defect. This document records the selected design, not a claim that validation has passed.

## Scope and decisions

1. Move the gutter icon into ShowPreviewAction as private state. The contributor reuses the action presentation icon. Delete PreviewIcons; preserve both SVG resources and the XML tool-window reference. Root-package relocation and a new public icon facade add inappropriate layout/API surface.
2. Preserve Compilation.sourceSets as visible/associated sources. Add compiledSourceSets for ownership, populated from each compilation's own Kotlin source sets. A five-argument compatibility constructor preserves existing callers without adding a data-class default. Compose checks consider only owning compilations, preserving the requirement that every compilation of a shared KMP source supports Compose. Update the metadata transport coherently and test round trips. Do not globally drop associated main sources: TestMirror and package-cycle checks depend on them.
3. Replace independent browser observation/navigation with observePages(urls). Register native handlers before starting URL collection. Use callbackFlow for registration cleanup, with one local event-processing coroutine so callbacks never mutate coroutine jobs across threads. Cancel pending title identification on navigation, clear, new main-frame load and failure. Stamp internally queued identification results to reject results belonging to an earlier navigation. This does not identify arbitrary delayed native callbacks without a native request ID. Preserve manual reload, hidden-window URL freezing, developer tools and disposal. Unsupported browsers emit directly from the URL stream. Avoid a generic navigation engine, dependency bags or browser-global coroutine scope.
4. Retain narrow callback suppressions for synchronous lifecycle functions and the scoped platform bridge. Correct CallbackDeclaration diagnostics to acknowledge documented review exceptions; do not exempt arbitrary names or whole platform units.
5. Retain LaunchDevServerUseCase's LongParameterList exception. Its collaborators and timeouts describe one process lifecycle; artificial bags or split services hide dependencies or complicate cancellation.
6. Own ServerSocket inside each test's use block; release it before testing a free port. Preserve all existing test names and expectations, remove redundant shared cleanup/suppression.

The general TrivialLambdaReference defect is included: a bound reference captures its receiver earlier than the original lambda. Report only when semantic analysis establishes a stable receiver, preserving mutable, delegated, computed or unresolved receiver reads. Unbound references using the lambda's receiver parameter remain eligible. Do not special-case Compose or rememberUpdatedState by spelling.

## Alternatives and review

Sol and Astra independently evaluated the choices, then challenged each other's lifetime, metadata and API assumptions. A subscription-only browser rewrite leaves stale title classification after navigation. Scattered atomics complicate compound state; a local event owner is simpler. An actor plus global mutex/navigation framework is unnecessary. For Compose, ignoring test compilations or accepting any Compose-enabled compilation would hide real KMP errors. For function references, a syntactic val check does not prove stable evaluation.

## Implementation sequence

1. Add regression examples preserving existing tests. Confirm the relevant failures before changing rules.
2. Implement metadata ownership and callback diagnostic (Sol); receiver semantics and icon/socket edits (root); browser lifetime and tests (Astra).
3. Review each other's implementation against this design and repository docs, especially cancellation, source ownership and test realism.
4. Run all linter tests/checks. Publish a new patch locally; update Kaleidos to that patch and remove its Compose workaround.
5. Run Kaleidos architectureLint, test and plugin distribution build with CI-equivalent arguments. Inspect the changed source/package layout and commit logical changes. Do not push or publish to Marketplace.

## Verification

- Compose: associated non-Compose test cannot veto main; genuinely shared sources compiled by mixed Compose/non-Compose targets remain rejected; unknown ownership and serialized metadata; existing TestMirror, cycle and platform checks.
- References: mutable locals/properties, delegated reads, custom/open getters, stable locals/parameters/objects and ordinary properties; old unbound, suspend, overload and arity examples unchanged.
- Browser: immediate load completion, A to B with pending/queued identification, clear, HTTP/transport errors, same-URL reload, title timeout, collection cancellation, partial registration failure, disposal and unsupported browser. Tests exercise production session logic through the native boundary, not a reimplementation in a fake.
- Socket: all four original cases retain their assertions.

Existing user edits (.gitignore in the linter checkout and untracked gradle/gradle-daemon-jvm.properties in Kaleidos) are outside this task. Native browser behaviour that cannot be exercised in a headless test must be reported as a verification limit rather than claimed as tested.

## Implemented result

Sol reviewed the design and receiver changes; Astra independently reviewed all six decisions. The browser implementation uses synchronous native registration hooks inside a collection-owned Flow extension rather than introducing a driver interface or another build unit. The Completion subtype restricts failure conversion to actual completion events. Scoped lifecycle/bridge callback exceptions and the existing use-case constructor exception remain unchanged.

Validation on 2026-11-04:

- Linter: `./gradlew check koverVerify detekt :detekt:koverXmlReport --console=plain` passed. 683 tests, zero failures/errors/skips. All configured coverage gates passed. Version 0.24.3 was published to Maven Local by the architecture self-check.
- Kaleidos: `./gradlew architectureLint test :plugin:buildPlugin --no-configure-on-demand --init-script /private/tmp/kaleidos-local-rules.init.gradle --console=plain` passed. 255 tests, zero failures/errors/skips, including 16 new browser tests. All four existing socket test names and assertions are retained.
- Temporary Kover instrumentation (not a committed build change) measured the new browser session/helper logic at 63/63 lines and 58/58 branches. Native JCEF calls and the composed IDE UI were not exercised inside a running IDE; their compilation and packaging passed.
- The local init script adds Maven Local while preserving default plugin repositories. No persistent repository configuration or CI workflow was changed during this phase. Remote CI needs rules 0.24.3 published before it can resolve the updated catalog.
- Marketplace publishing remains disabled by the earlier Kaleidos commit. No pushes or remote releases were performed.

## Package and ownership review

- T1: browser behavior stays in the existing IntelliJ UI unit; socket tests stay in server/api. No new sub-context or build unit.
- T2: no new use case or use-case interface.
- T3: immutable browser events are in browser/model; scoped Flow and conversion extensions are in browser/util.
- T4: existing Compose kinds and the existing browser/component implementation placement are retained.
- T5: no new operation interface; synchronous registration hooks are scoped to the existing platform bridge.
- T6: one sealed event family per model file; extension files are named after their receivers.
- T7: existing DI wiring and project/tool-window lifecycle ownership are retained.
- T8: browser tests mirror their behavior owners; the scripted native registration double is in test browser/fake.

The approved icon relocation deletes the old public theme container and uses the action's existing presentation; no declaration was promoted into another API unit. Both worktree indexes were refreshed after source changes.
