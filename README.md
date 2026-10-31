[![version](https://img.shields.io/maven-central/v/ru.astrainteractive.klibs.rules/detekt?style=flat-square)](https://github.com/Astra-Interactive/rules-architecture)
[![kotlin_version](https://img.shields.io/badge/kotlin-2.4.10-blueviolet?style=flat-square)](https://github.com/Astra-Interactive/rules-architecture)
[![detekt_version](https://img.shields.io/badge/detekt-2.0.0--alpha.6-orange?style=flat-square)](https://detekt.dev)

## Rules Architecture

Rules Architecture turns the package-layout and architecture rules of our Kotlin projects into checks that
fail the build. A rule that no tool checks decays, so instead of trusting a document, the checks read your
sources and tell you which file breaks which rule and how to fix it.

## ✨ Features

- **Per-file rules for detekt**: 64 custom rules plus settings for detekt's own rules, with IDE support and
  `@Suppress`.
- **Cross-file checks with Konsist**: role import direction, use case shape, contract and implementation
  levels, DI placement, test mirroring and more.
- **Unit structure checks**: the unit graph (`api` never depends on `impl`), one-sentence unit READMEs, no unit
  nested in another.
- **Gradle plugins**: one plugin that applies detekt with the rules and runs the Konsist and structure checks
  of every unit in `check`, with a baseline for old code.
- **Multiplatform**: the detekt rules read syntax only and the Konsist checks parse source directories, so both
  work on `commonMain` as well as on JVM sources.

## 🚀 Installation

Five artifacts, one version:

| Artifact                                       | Use it for                                                      |
|------------------------------------------------|-----------------------------------------------------------------|
| `ru.astrainteractive.klibs.rules:gradle-plugin` | Gradle plugins that wire the three artifacts below into a build |
| `ru.astrainteractive.klibs.rules:detekt`       | detekt plugin with the rules and their settings                 |
| `ru.astrainteractive.klibs.rules:konsist`      | Konsist checks, run from the command line                       |
| `ru.astrainteractive.klibs.rules:structure`    | structure checks, called from your build logic                  |
| `ru.astrainteractive.klibs.rules:core`         | the shared vocabulary of the checks, pulled in by the others    |

#### Version catalogs

```toml
[versions]
klibs-rules = "<latest-version>"

[libraries]
klibs-rules-detekt = { module = "ru.astrainteractive.klibs.rules:detekt", version.ref = "klibs-rules" }
klibs-rules-konsist = { module = "ru.astrainteractive.klibs.rules:konsist", version.ref = "klibs-rules" }
klibs-rules-structure = { module = "ru.astrainteractive.klibs.rules:structure", version.ref = "klibs-rules" }
```

## 🛠️ Basic Usage

### Gradle plugins

Two plugins, both from `ru.astrainteractive.klibs.rules:gradle-plugin`, published to Maven Central. Add
`mavenCentral()` to the `pluginManagement` repositories, then apply a plugin to every unit. Apply the linter to the
root project too when using `architectureGraphCheck`:

```kotlin
plugins {
    id("ru.astrainteractive.klibs.rules.gradle.linter") version "<version>"
}
```

| Plugin id                                     | Does                                                                              |
|-----------------------------------------------|-----------------------------------------------------------------------------------|
| `ru.astrainteractive.klibs.rules.gradle.detekt` | applies detekt with a bundled `detekt.yml`, the ktlint wrapper and the compose rules |
| `ru.astrainteractive.klibs.rules.gradle.linter` | applies the detekt plugin and the architecture rules, and adds `architectureCheck`   |

**Detekt plugin.**

- Applies `dev.detekt` and registers a `detektFormat` task with auto-correction on.
- Writes the bundled `detekt.yml` to the root project's `build/detekt` once for the whole build, and enables
  HTML reports, parallel execution and `allRules`. The project's own detekt config (`config/detekt/detekt.yml`
  or `detekt { config }`) is read after the bundled one, so its settings win.
- Resolves types against the Kotlin JVM target of the project, or the JVM that runs Gradle when there is none.
  Set another target with `detektRules { jvmTarget.set("21") }`.
- `detekt` and `detektFormat` read every `.kt` and `.kts` file of the project directory; the source-set tasks
  (`detektMain`, `detektTest`, …) read only their source set, so each is resolved against its own classpath. No
  task enters a directory that is a build or a checkout of its own (a Gradle build or settings file, or a `.git`
  entry, such as a worktree under `.claude/worktrees`), nor `build` and `resources`; sources generated into a
  project's `build` directory (moko-resources) are skipped by the source-set tasks too.

**Linter plugin.** Apply it instead of the detekt plugin.

- Adds the architecture detekt rules and their `detekt-architecture.yml`, listed after `detekt.yml` and before the
  project's own config, and passes the module's explicit API mode to the `detekt` task.
- Registers `architectureCheck`, which `check` depends on: the Konsist checks of the unit's package layout, run in
  a separate JVM, and the structure checks of its README, nested units and project dependencies. Konsist findings
  accept `@Suppress("RuleName")` on a declaration or `@file:Suppress("RuleName")` on a whole file.
- Registers `architectureBaseline`, which writes the current violations to `architecture-baseline.txt`; the check
  accepts what the file lists for findings that cannot carry a Kotlin suppression.
- On the root project, registers `architectureGraphCheck`, which checks the complete unit graph for dependency
  cycles and for dependencies on an entry point, a unit detected or set as `entry-point` (an `application`
  plugin, an `instances` path); root `check` depends on it. Units contribute their dependencies through an isolated-projects-compatible
  shared build service.

The unit kind decides which layout rules apply. It is detected by convention and can be set:

```kotlin
linter {
    unitKind.set("library") // context, library, api, contract, entry-point or technology
    rootPackage.set("ru.astrainteractive.aspekt.module.jail")
    // The project's own words, added to those of the package-layout rule
    entryPointWords.addAll("slot", "panel")
    technologyWords.add("mongo")
    pluralConcepts.add("goods")
    // Supertypes of classes the platform constructs, added to Application, Activity, Service, Worker and the others
    platformClasses.add("WatchFaceService")
}
```

The technology words also come from the unit's build file: the last segment of each library's group and the first
word of its artifact (`org.mongodb:mongodb-driver-kotlin-coroutine` gives `mongodb`), generic words such as `core`
and `runtime` aside. The Konsist checks and the detekt rules that judge technology packages (`BareIdentifier`,
`KindNameInPackage`, `RunBlockingOutsideBoundary`) get the same words; `architectureDetektSettings` writes them into
`build/detekt/architecture-settings.yml`, read before the project's own detekt config, together with whether
`RunCatchingCancellation` accepts the inline `CancellationException` check: only when mikro-core is not in the
project's version catalog. A convention plugin sets
`unitKind` for the units it builds, which maps each plugin to a row of the unit direction table.

| Detected as       | When                                                                                     |
|-------------------|------------------------------------------------------------------------------------------|
| `api`, `contract` | the last segment of the project path                                                     |
| `technology`      | a technology name as the last segment of the project path (`data/exposed`, `data/room`, `data/memory`), or a platform name outside `core` and the entry points (`gui/bukkit`) |
| `entry-point`     | the `application` or `com.android.application` plugin, or a path under `:instances`      |
| `library`         | Kotlin explicit API mode strict in a unit applying `maven-publish`                       |
| `context`         | anything else                                                                            |

The plugins use the rules of their own version, so a version of the plugin brings the same version of the
`detekt`, `konsist` and `structure` artifacts. The sections below are for builds that wire the artifacts by hand.

### detekt rules

Add the rules to a project that applies detekt, together with the settings of detekt's own rules. Copy
[`detekt-architecture.yml`](rules-kotlin/detekt/src/main/resources/detekt-architecture.yml) into the project and
list it after your own configuration, because a later file overrides an earlier one:

```kotlin
dependencies {
    detektPlugins("ru.astrainteractive.klibs.rules:detekt:<version>")
    // the settings configure ktlint's ImportOrdering
    detektPlugins("dev.detekt:detekt-rules-ktlint-wrapper:<detekt-version>")
}

detekt {
    // the rules switch themselves on in the jar's config/config.yml, which detekt reads only on top of its defaults
    buildUponDefaultConfig.set(true)
    config.from(files("config/detekt/detekt.yml", "config/detekt/detekt-architecture.yml"))
}
```

The rules run on `./gradlew detekt` and accept `@Suppress("RuleName")`.

`RedundantVisibilityModifier` must know the explicit API mode of a library unit, and detekt passes it only to
its per-source-set tasks. In a unit with `explicitApi()`, pass it to the plain `detekt` task too:

```kotlin
tasks.withType<dev.detekt.gradle.Detekt>().configureEach {
    freeCompilerArgs.add("-Xexplicit-api=strict")
}
```

For old code, detekt's own baseline accepts today's findings: `./gradlew detektBaseline` writes
`detekt-baseline.xml`, which `detekt` then reads.

### Konsist checks

The Konsist checks run in their own JVM, because Konsist embeds the Kotlin compiler. Its command-line entry
point, `ru.astrainteractive.klibs.rules.konsist.cli.KonsistCli`, checks one unit and writes one violation per
line as `rule<TAB>path<TAB>subject<TAB>message`; it finishes normally either way, so the caller decides what
fails the build. A task in the unit's `build.gradle.kts`:

```kotlin
val konsist by configurations.creating

dependencies {
    konsist("ru.astrainteractive.klibs.rules:konsist:<version>")
}

val konsistReport = layout.buildDirectory.file("reports/konsist.tsv")
val unitPath = path

val konsistCheck = tasks.register<JavaExec>("konsistCheck") {
    val reportFile = konsistReport.get().asFile
    classpath = konsist
    mainClass.set("ru.astrainteractive.klibs.rules.konsist.cli.KonsistCli")
    args(
        "--project-dir", projectDir.absolutePath,
        "--unit-kind", "context",
        "--unit-path", unitPath,
        "--main", file("src/main/kotlin").absolutePath,
        "--test", file("src/test/kotlin").absolutePath,
        "--report", reportFile.absolutePath,
    )
    doLast {
        val violations = reportFile.readLines()
        check(violations.isEmpty()) { violations.joinToString(separator = "\n") }
    }
}

tasks.named("check") { dependsOn(konsistCheck) }
```

| Option           | Value                                                                              |
|------------------|------------------------------------------------------------------------------------|
| `--project-dir`  | the unit's directory; report paths are relative to it                              |
| `--unit-kind`    | `context`, `library`, `api`, `contract`, `entry-point` or `technology`, see below  |
| `--unit-path`    | the unit's Gradle path; finds the root package (`:modules:event-bukkit` → `…event`) |
| `--main`         | a main source directory, optionally `sourceSet=path`; repeat for every source set |
| `--test`         | a test source directory, optionally `sourceSet=path`; repeatable                  |
| `--report`       | the file the violations are written to                                             |
| `--root-package` | the unit's root package, when the unit path does not find it                       |

Suppress a rule in Kotlin instead of adding its finding to a baseline. Use a declaration annotation for one
class, interface, function or property, including all declarations nested in it:

```kotlin
@Suppress("ContractName")
interface UserDao
```

Use a file annotation for every finding of that rule in the file, including import, package and file-shape rules:

```kotlin
@file:Suppress("KindShape", "PlatformImport")

package com.example.feature
```

Suppression applies to the named rules only. `architectureBaseline` remains available for structure and Gradle
build findings, and for findings about missing files or directories that cannot carry an annotation.

Every unit has the shape `<unit root>[/<concept>]/<kind>/<File>`. The unit kind decides the exceptions:

| Kind          | The unit                                                                                  |
|---------------|-------------------------------------------------------------------------------------------|
| `context`     | is a sub-context unit (`command`, `event-bukkit`, `gui/bukkit`)                           |
| `library`     | is a published library or `core`                                                          |
| `api`         | holds platform-free code other units call (`gui/api`); no file imports a platform type   |
| `contract`    | holds the wire types a server and its client share                                        |
| `entry-point` | holds the platform main class, which may sit at the root, and `RootModule`                |
| `technology`  | is wholly one technology's implementation (`data/exposed`); its concepts hold implementations directly |

### Structure checks

The structure checks need the project's dependencies, so they are a plain Kotlin library for build logic,
such as a convention plugin. Describe the unit and run the checks:

```kotlin
val checker = StructureChecker(listOf(UnitDependency(), UnitReadme(), NestedUnit(), RootSources()))

val violations: List<StructureViolation> = checker.check(
    BuildUnit(
        path = project.path,
        directory = project.projectDir,
        isEntryPoint = false,
        isRoot = project.path == ":",
        projectDependencies = setOf(":economy:api")
    )
)
```

## 🔍 What Is Checked

### detekt rules

| Rule                           | Reports                                                                          |
|--------------------------------|----------------------------------------------------------------------------------|
| `ImplicitItParameter`          | the implicit lambda parameter `it`                                               |
| `AnonymousTuple`               | `Pair` and `Triple` used as a type, constructed by name, or made with `to` outside `mapOf(…)` |
| `DataClassDefaultValue`        | a default value in a data class that is not `@Serializable`                      |
| `ConstructorDefaultDependency` | a constructor default that creates a dependency, such as `= Foo()`               |
| `TestFunctionName`             | a `@Test` not named `GIVEN_<state>_WHEN_<action>_THEN_<outcome>`                 |
| `CalleeBeforeCaller`           | a function that calls a function declared below it in the same class or file     |
| `DeclarationOrder`             | properties, secondary constructors, functions, `init` blocks, companion: out of this order; a top-level `PREVIEW_*`/`Preview*` fixture may follow the composables; anything after the companion object, nested types included |
| `NamespaceObject`              | an `object` that only groups functions, except a Compose `<Component>Defaults`   |
| `LocalOrInnerClass`            | a class inside a function body and an `inner class`                              |
| `TimeAsPrimitive`              | a time-named value (`createdAt`, `timeoutMs`, `CARD_IN_MS`) typed as a number or a string, or inferred from such a literal |
| `KDocHtmlTag`                  | Javadoc HTML (`<p>`, `<code>`) in KDoc                                           |
| `ImplementationName`           | the `Impl` and `Interface` suffixes and the `I` prefix on an interface           |
| `KindNameInPackage`            | `Service` outside a `service` package, `Model`/`Entity` in a `model` with no technology word (`exposed`, `room`, ...) before it, `Helper`/`Util`/`Manager` names |
| `ExpressionBodyOnOneLine`      | an expression-body function whose signature and body are not one line            |
| `BooleanOperatorWrapping` | a line break on either side of `&&` or `\|\|` |
| `ArgumentPerLine` | a call or constructor call with three or more arguments, a trailing lambda included, whose arguments in parentheses are not each on its own line |
| `PropertyReadBeforeDeclaration` | a property read while the object is constructed (an initializer, a delegate, an `init` block) before its declaration; lambdas, functions and `const val` are not judged; replaces detekt's `PropertyUsedBeforeDeclaration` |
| `GuardIfBraces` | an `if` without `else` whose body is one `return` in braces, when it fits one line of 120 columns |
| `EventCollection` | one-off `events` collected in `LaunchedEffect` without `withContext(Dispatchers.Main.immediate)`, or elsewhere without `repeatOnLifecycle` |
| `CompletableDeferredUse` | a `CompletableDeferred` created and awaited in one function, or held in a property as a `Unit`/`Boolean` readiness flag |
| `BridgeCleanup` | an `awaitClose` or `invokeOnCancellation` cleanup that assigns state declared outside it or calls `launch` |
| `ExplicitInvoke` | a call `name(…)` of a parameter, property or local value instead of `name.invoke(…)` |
| `FunctionVerb` | a function whose first word is `get` (but a view model's `getState`), `map`, `make`, `do`, `handle` or `process` |
| `PropertyAsFunction` | a function with no parameters that only returns a property of its class or file |
| `OneTypeFamilyPerFile` | a top-level function (a composable included) beside a top-level type, and a second class with behaviour in one file |
| `FileLength` | a file that passes 400 lines |
| `ConstantOutsideCompanion` | a `const val` or literal `val` in a class body outside its companion, a `private object` of constants beside a type |
| `ComposableFileType` | a class, interface or object in a file with a `@Composable` function, except a `*Defaults` object |
| `ComposableName` | a `@Composable` that emits UI and whose name does not end in `Composable`, except previews, the theme function and an override such as `Render()`; a `…ScreenComposable` in a file of another name; a theme function outside `<Theme>Composable.kt` |
| `ThemeObjectFunction` | a function inside an object of a `theme` package |
| `ComposeConstantCase` | a `const val` not `UPPER_SNAKE` or a `val` not `PascalCase` in a composable or `theme` file |
| `PreviewBlockLast` | a non-preview declaration after the first `@Preview`, a preview helper above the code of the file, a `@Preview` not named `…Preview` |
| `RememberInName` | a `remember…` function whose body calls no `remember` |
| `ActualFileSuffix` | a file with an `actual` declaration whose name has no platform suffix (`Name.android.kt`) |
| `MutableStateFlowEmit` | `emit`/`tryEmit` on a property declared or initialised as a `MutableStateFlow` |
| `ImpureUpdateLambda` | an assignment to an outer name inside the lambda of `update`, `getAndUpdate` or `updateAndGet` |
| `SingleJobLaunchMode` | a launch on a single-job scope without a `SingleJobMode`, an unguarded launch in an `on…` handler of a view model or component |
| `StateFlowTimeout` | `StateFlow.first()` inside `withTimeout` or `withTimeoutOrNull` |
| `ThrowInsideRunCatching` | a `throw` inside the lambda of `runCatching`, `mapCatching` or `recoverCatching` |
| `RunCatchingCancellation` | a `runCatching` in a `suspend` function or coroutine lambda not followed by `.propagateCancellationException()` or the inline `onFailure` check for `CancellationException` |
| `ResultConsumption` | a `Result` discarded as a statement without consuming its success or failure |
| `GetOrThrowInMain` | `getOrThrow()` outside the test sources |
| `BareIdentifier` | a property named `id`/`…Id` typed `String`, `Int`, `Long` or `UUID` outside a `@Serializable` class or technology package, a `…Id` class that is not a value class |
| `CompanionFactoryFunction` | a `create…` function in a companion object |
| `ClockSystemOutsideDi` | `Clock.System` outside a `di` package and the test sources |
| `RolelessClassName` | a `…Validator` or `…Factory` class (a Decompose `fun interface Factory` and a type named like the supertype it implements, `object Factory : PageContext.Factory`, are allowed), `…WarmUp`/`…Migration`/`…ViewModel` outside its `warmup`/`migration`/`viewmodel` package |
| `DiModuleLifecycle` | a `…Module` that declares `enable`, `disable`, `reload`, `onEnable`, `onDisable` or `onReload`, or that starts a warm-up, migration, listener, task, service or `launch` without a `lifecycle` property |
| `RootModuleConstruction` | a `RootModule` constructor call that constructs anything but a DI module (a class of a `di` package, found through the call's import) or a `Lifecycle` |
| `SystemTimeCall` | `System.currentTimeMillis()` and `System.nanoTime()` |
| `RunBlockingInTest` | `runBlocking` in a test source set |
| `RunTestResult` | a `@Test` that calls `runTest` without declaring `TestResult` and returning `runTest { … }` from a block body; `= runTest { }` is reported too |
| `RunBlockingOutsideBoundary` | `runBlocking` in the main sources outside a top-level `main` function and outside a technology package |
| `ThreadPrimitive` | `Thread` constructed or used through its static members in the main sources |
| `CallbackFlowShape` | a `callbackFlow` whose block does not end with `awaitClose` (it throws at run time), `launch { send }` in a callback of it (not in the producer block itself), a rendezvous `buffer`, a bridge shared with `shareIn`/`stateIn` not `WhileSubscribed()` |
| `ChannelFlowShape` | a `channelFlow` that registers a listener without ending in `awaitClose`, or launches no coroutine of its own |
| `SharedFlowEvent` | a `SharedFlow` property of a view model, a `MutableSharedFlow` without `extraBufferCapacity` fed through `tryEmit` |
| `CallbackDeclaration` | an `on…` parameter or property of a function type outside a `@Composable` and a `Modifier` factory, an `…Listener` interface, an `…Callback` interface other than a Decompose `fun interface On…Callback`; a top-level `On…Callback` output not nested in its component |
| `KDocFiller` | a KDoc filler phrase, a `@return` of "the result" or of the return type's own words, a `@param` whose words only repeat its name, a summary of more than four sentences |
| `LambdaSingleIfGuard` | a lambda whose whole body is one `if` without `else`, unless the `if` only throws |
| `OperatorBeforeTrailingCall` | arithmetic in parentheses before a call or property (`(a * b).sp`), a chain of `plus`/`minus`/`times`/`div`/`rem` calls |
| `TrivialLambdaReference` | a lambda that only passes its parameters to one function, or a `{ obj.foo() }` callback, instead of a function reference; in coroutine code only when the callee resolves to a non-`suspend` function |
| `ComposableWhenBranch` | a `when` branch of a `@Composable` function that spans several lines, except a block of several statements |
| `NonComposableFileInComposable` | a file in a `composable` package with no `@Composable` function, outside test sources |
| `DurationOverloadCall` | `.inWholeMilliseconds`/`.inWholeSeconds`/`.inWholeNanoseconds` passed to `delay`, `withTimeout`, `debounce` and other calls with a `Duration` overload |

### detekt settings

`detekt-architecture.yml` configures detekt's own rules: `ClassOrdering` off, replaced by `DeclarationOrder`,
which puts `init` blocks after the functions they call; `ForbiddenComment` for region
markers and separator comments; `ForbiddenImport` for `kotlinx.datetime.Instant`, `kotlinx.datetime.Clock` and `java.util.Date`, for `java.time` types and `java.util.Calendar` outside technology packages (`ForbiddenImport/time`), and for platform and ORM
types in `model`, Compose excepted, outside technology packages (`exposed`, `room`, ...); `RedundantVisibilityModifier`; `WildcardImport` with no exceptions; ktlint `ImportOrdering`.
`DataClassContainsFunctions`, with no conversion exception, and `DataClassShouldBeImmutable`: a data class
holds fields only. `GlobalCoroutineUsage`: no global scope. `PackageNaming` with a lowercase-only pattern.
`MagicNumber` accepts a number in a property declaration, where the Compose rule names a value.
`ForbiddenImport` also bans DI frameworks (Koin, Dagger, Anvil, Metro, kotlin-inject, Kodein) and
the callback and thread tools of the no-callbacks rule: `suspendCoroutine`, `trySendBlocking`, `Timer`, `TimerTask`,
`Handler`, `thread`; `Executor`/`Executors` outside `core` (`ForbiddenImport/executor`) and `CompletableFuture`
outside technology packages (`ForbiddenImport/future`), and Compose's `painterResource` and `stringResource`
outside `util`, where the `asPainter()` and `asComposableString()` extensions wrap them
(`ForbiddenImport/resource`). `SuspendFunSwallowedCancellation` and
`SuspendFunInFinallySection` are active.

### Konsist checks

| Check                  | Reports                                                                                                   |
|------------------------|-----------------------------------------------------------------------------------------------------------|
| `KindShape`            | a file loose in the unit root or a concept package, a package below a kind package (`theme` included); below `composable`, a second level, a `model`/`util`/`theme` sub-package, or files next to sub-packages |
| `RootPackagePath` | a root package that does not end with the unit name from its path, outside the entry point and libraries; a `core` unit may end at `core`, and the last segment may be the singular of a plural unit name |
| `ThemeVisibility` | a `theme` object that is not `internal` in a feature unit, or hidden in the design-system unit `core/ui` |
| `AnonymousProjectObject` | an anonymous `object :` implementing a type of the project instead of a library type |
| `KindImportDirection`  | an import against the first table of `arch-context.md`: `model` and contracts depend on `model` only, implementations on contracts, `model`, `util` and their own technology packages, entry points on use cases, contracts, `model` and `util`, one entry point on another only between UI kinds, a router's implementation on the UI it opens, a screen only from what hosts or opens it, nothing on `di` |
| `PlatformImport`       | a platform or ORM import in `model`, `api` or `usecase`, or anywhere in an `api` unit; the `model` of a UI unit (a file outside `model` imports Compose) may import Compose |
| `UseCaseShape`         | `*UseCase` outside `usecase`, other names in `usecase`, more than one entry, a use case taking another, a pass-through `invoke` that is one call on one of its properties |
| `ContractPlacement`    | a plain interface outside `api` (a use case's in `usecase` and a DI module's in `di` aside), a state holder's interface (`…Component`, `…ViewModel`) outside `component`/`viewmodel`, `api` included, an implementation inside `api`, a `network`/`local`/`remote` package; a Compose scope (`: ColumnScope`) is not a contract |
| `ForbiddenPackageName` | a forbidden word in a package created inside the unit; the root package may keep its unit's word; `table`, `entity` and `database` outside a technology unit or package |
| `UtilShape`            | a class or object in `util`, an extension function outside `util`, its receiver's file and entry-point kinds included, except a `private` one in a composable file; names `util/<Subject>Ext.kt`, the element type for a collection |
| `EntryPointVisibility` | a public class in a kind the platform invokes (`command`, `event`, `expansion`, `route`, `service`, `warmup`, ...), outside a `library` unit |
| `ErrorPlacement`       | an error type outside a `model` package                                                                   |
| `DiModuleVisibility` | a public DI module that a file of its own unit constructs, outside a library |
| `DiPlacement`          | a `*Module` outside a `di` package                                                                        |
| `NestedDataClass`      | a top-level data class used as a property type by exactly one class, except a sealed variant or a `*State` |
| `TestMirror`           | a test in another package or source set than its owner                                                    |
| `FakePlacement`        | a `Fake*` outside a `fake` package of the tests, or in the main sources outside a `<feature>/fake` unit, and a test fake of another unit's contract outside `testFixtures`, and a class other than a fake in a `<feature>/fake` unit |
| `InMemoryPlacement`    | an `InMemory*` outside a `memory` package of the main sources                                              |
| `SealedFamilyFile` | a class or object whose direct supertype is a sealed type declared in another file |
| `StateTypePlacement`   | a `<Screen>State` declared in a UI kind (a state holder, a composable, a menu), nested in a type there or beside it, instead of its own file in `model` |
| `NestedEnum` | a top-level enum used as the property type of exactly one class |
| `UtilFileName` | a `util` file of one receiver's extensions not named `<Receiver>Ext.kt`, a file named `Utils.kt`, `Helpers.kt`, `Extensions.kt` and the like |
| `UtilFunctionAsExtension` | a `util` function whose one parameter is a model of the unit or of a unit it depends on, instead of an extension |
| `ImplementationVisibility` | a public class in a technology package or a technology unit's concept, outside a library, `core`, `stub` and `memory` |
| `UseCaseAbstraction` | an interface or abstract class named `…UseCase` |
| `AndroidNamespace` | an Android unit whose `namespace` differs from its root package |
| `ApplicationKDoc` | KDoc outside a published library, other than a contract's failure case (`@return failure`, `@throws`) or a reason outside the code (a link, an issue key, a bug, workaround or quirk) |
| `ContractName` | an `api` interface named `…Api`, `…Dao`, `…Storage` or `…Krate`, a `…Repository` extending a generic repository or `Crud` base |
| `CoreShape` | a `…UseCase` class or a `…Repository` interface in a `core` unit |
| `RootModulePlacement` | a `RootModule` outside the entry-point unit or its `di` package |
| `EntryPointShape` | a file of an entry-point unit outside its root and its `di` package; a class the platform constructs (by its direct supertype) is sent to the root package, any other file out of the unit |
| `ContractUnitShape` | in a `contract` unit, a class or interface that is not `@Serializable`, a function with a body outside an object |
| `ComposeKindWithoutCompose` | a `composable` or `theme` kind package in a unit that imports no Compose; the root package and concepts are not judged |
| `PlatformPackage` | a package named after a platform (`android`, `ios`, `jvm`, …) in a unit with several main source sets |
| `PluralPackageName` | a plural package name below the unit root, except an inherently plural concept (`news`, `settings`) |
| `DuplicateRepresentation` | a technology data class with the same constructor parameter names as a `model` data class of the unit or of a unit it depends on |
| `FieldDependencyConstruction` | a behaviour class of the unit, or a library class named for a dependency's role (`HttpClient`, `…Repository`, `…Gateway`, `…Database`, `…DataSource`), constructed outside `di`; a value holder (no supertype, no function body, no `init` block) is not a dependency |
| `TestOwnerBehaviour` | a `<Owner>Test.kt` whose owner is a data class without a body or an enum without functions |

### Structure checks

| Check            | Reports                                                                                      |
|------------------|----------------------------------------------------------------------------------------------|
| `UnitDependency` | a project dependency outside the allowlist for the unit's kind in `arch-context.md`; every unit under `core/ui` counts as the design-system unit and may depend on the others, cycles left to `UnitDependencyCycle` |
| `UnitDependencyCycle` | a cycle in the complete project-unit graph (root Gradle plugin) |
| `PluginImplUnit` | an `impl` unit in a build that has a platform unit, a plugin (root Gradle plugin) |
| `UnitReadme`     | a unit without a `README.md` of one sentence                                                 |
| `NestedUnit`     | a unit inside the directory of another unit                                                  |
| `ContainerSources` | sources in a container directory above a unit (`modules/gui/src` above `modules/gui/api`) |
| `UnitPath` | a sub-context unit at the root of the build instead of a container such as `modules` |
| `PlatformSourceSetUnit` | a unit named after a Multiplatform target (`android`, `ios`, `jvm`, ...) instead of a source set |
| `RootSources`    | sources in the root project                                                                  |
| `EntryPointDependency` | a dependency on an entry-point unit (a path with the segment `instances`) |
| `TechnologyDependency` | a technology dependency forbidden by the unit direction table, including unrelated technology units |
| `ApiUnitDependency` | an `api` unit depending on a unit that is not `api`, `core` or a storage contracts unit (`dao`, `contract`) |
| `RootBuildSubprojects` | a `subprojects { }` or `allprojects { }` block in the root build file |
| `ProjectDirMatchesPath` | a unit whose directory differs from its Gradle path |
| `ApiConfiguration` | a directly declared `api` dependency that is not exported through a Multiplatform framework (Gradle plugin) |
| `PublishedLibraryExplicitApi` | a Kotlin unit applying `maven-publish` without strict explicit API mode (Gradle plugin) |
| `UnpublishedExplicitApi` | a Kotlin unit with explicit API mode that does not apply `maven-publish` (Gradle plugin) |
| `ApiUnitPlatformDependency` | an external platform dependency (Paper, Bukkit, Fabric, Forge, Android, `androidx` but Compose, annotation and collection) in an `api` unit (Gradle plugin) |

## 🗂️ Repository

| Path                     | Holds                                                                     |
|--------------------------|---------------------------------------------------------------------------|
| `rules-kotlin/detekt`      | the detekt rules, their `config/config.yml` and `detekt-architecture.yml` |
| `rules-kotlin/gradle-plugin` | the detekt and linter Gradle plugins                                    |
| `rules-kotlin/konsist`     | the Konsist checks and their command-line entry point                     |
| `rules-kotlin/structure`   | the structure checks                                                      |
| `rules-kotlin/self-check`  | the isolated build that runs the real linter on every unit and the graph   |

Build and test locally:

```shell
cd rules-kotlin
./gradlew check                # tests, 100% line coverage, detekt and architecture self-checks
./gradlew publishToMavenLocal  # ru.astrainteractive.klibs.rules:{detekt,gradle-plugin,konsist,structure}
```

`check` publishes the current artifacts to Maven Local, copies the sources into `self-check`, and runs
`architectureCheck` in every unit plus `architectureGraphCheck`, with no architecture baselines.

---


## 💖 Support the project

<table>
<tr>
<td align="center" width="130">
<img src="https://cdn.simpleicons.org/telegram/26A5E4" width="25" alt="Telegram"/><br/>
<sub><b>Telegram</b></sub>
</td>
<td align="center">
<a href="https://t.me/makeevrserg">
<img width="70%" src="https://img.shields.io/badge/@makeevrserg-Message-26A5E4?style=for-the-badge&logo=telegram&logoColor=white" alt="Telegram @makeevrserg"/>
</a>
</td>
</tr>
<tr>
<td align="center" width="130">
<img src="https://cdn.simpleicons.org/boosty/F15F2C" width="25" alt="Boosty"/><br/>
<sub><b>Boosty</b></sub>
</td>
<td align="center">
<a href="https://boosty.to/empireprojekt/donate">
<img width="70%" src="https://img.shields.io/badge/Donate-Boosty-F15F2C?style=for-the-badge&logo=boosty&logoColor=white" alt="Donate on Boosty"/>
</a>
</td>
</tr>
<tr>
<td align="center" width="130">
<img src="https://cdn.simpleicons.org/tether/50AF95" width="25" alt="USDT"/><br/>
<sub><b>USDT</b></sub><br/>
<sub>Polygon (POL)</sub>
</td>
<td>

```text
0x3955abc6f5396e57b11a05b96f988bc60708c9b0
```

</td>
</tr>
<tr>
<td align="center" width="130">
<img src="https://cdn.simpleicons.org/tether/50AF95" width="25" alt="USDT"/><br/>
<sub><b>USDT</b></sub><br/>
<sub>TRC20</sub>
</td>
<td>

```text
TLYf28vZeuuHcEJMHSZtuYEzQ2DjNvNE3W
```

</td>
</tr>
<tr>
<td align="center" width="130">
<img src="https://cdn.simpleicons.org/tether/50AF95" width="25" alt="USDT"/><br/>
<sub><b>USDT</b></sub><br/>
<sub>Solana</sub>
</td>
<td>

```text
6sYK6Nss8cjLeeTp6u3t63JG4f1hFf8sDQ7nVtMNoyps
```

</td>
</tr>
<tr>
<td align="center" width="130">
<img src="https://cdn.simpleicons.org/ton/0098EA" width="25" alt="TON"/><br/>
<sub><b>TON</b></sub><br/>
<sub>TON network</sub>
</td>
<td>

```text
UQDfywxsnHI1ko_uqBYKED3RoMzoVm3mnxuS_-JVQc4mSSJt
```

</td>
</tr>
</table>

A ⭐ on the repository costs nothing and helps just as much.