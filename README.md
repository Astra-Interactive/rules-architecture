[![maven_central](https://img.shields.io/maven-central/v/ru.astrainteractive.klibs.rules/detekt?style=flat-square&label=maven%20central)](https://central.sonatype.com/namespace/ru.astrainteractive.klibs.rules)
[![crates_io](https://img.shields.io/crates/v/klibs-rules?style=flat-square)](https://crates.io/crates/klibs-rules)
[![kotlin_version](https://img.shields.io/badge/kotlin-2.4.10-blueviolet?style=flat-square)](https://github.com/Astra-Interactive/rules-architecture)
[![detekt_version](https://img.shields.io/badge/detekt-2.0.0--alpha.6-orange?style=flat-square)](https://detekt.dev)
[![rust_version](https://img.shields.io/crates/msrv/klibs-rules?style=flat-square&label=rust)](https://crates.io/crates/klibs-rules)

## Rules Architecture

A linter for the package layout and architecture of Kotlin, Kotlin Multiplatform and Rust projects. Both linters
enforce the same architecture: units and their roles, the direction of dependencies, which kind of code lives
where, and naming.

|          | [Kotlin](#kotlin)                                      | [Rust](#rust)                                               |
|----------|--------------------------------------------------------|-------------------------------------------------------------|
| Sources  | [`rules-kotlin`](rules-kotlin)                         | [`rules-rust`](rules-rust)                                  |
| Ships as | Gradle plugins and libraries on Maven Central          | the `klibs-rules` crate on crates.io                        |
| Runs as  | `./gradlew architectureLint`                           | `cargo klibs-rules`                                         |
| Made of  | detekt rules, Konsist checks, Gradle structure checks  | a required rustc, clippy and rustfmt configuration and rules of its own |

## Kotlin

detekt rules, Konsist checks and Gradle structure checks behind one Gradle plugin.

### 🚀 Installation

```toml
[versions]
klibs-rules = "<latest-version>"

[libraries]
# Needed only to wire the checks by hand, without the plugins (see "Without the Gradle plugin")
# detekt plugin with the rules
klibs-rules-detekt = { module = "ru.astrainteractive.klibs.rules:detekt", version.ref = "klibs-rules" }
# Konsist checks, run from the command line
klibs-rules-konsist = { module = "ru.astrainteractive.klibs.rules:konsist", version.ref = "klibs-rules" }
# Structure checks, called from build logic
klibs-rules-structure = { module = "ru.astrainteractive.klibs.rules:structure", version.ref = "klibs-rules" }

[plugins]
# Everything: detekt with the rules below, the Konsist and structure checks, the architecture tasks
klibs-rules-linter = { id = "ru.astrainteractive.klibs.rules.gradle.linter", version.ref = "klibs-rules" }
# Only detekt with the bundled detekt.yml, the ktlint wrapper and the Compose rules
klibs-rules-detekt = { id = "ru.astrainteractive.klibs.rules.gradle.detekt", version.ref = "klibs-rules" }
```

The plugins are on Maven Central: add `mavenCentral()` to the `pluginManagement` repositories.

### 🛠️ Configuration

Apply the linter to the root project and to every unit.

```kotlin
// root build.gradle.kts
import ru.astrainteractive.klibs.rules.core.unit.model.ProductKind

plugins {
    alias(libs.plugins.klibs.rules.linter)
}

linter {
    productKind(ProductKind.APP) // APP or PLATFORM_PLUGIN; every unit inherits it
}
```

```kotlin
// a unit's build.gradle.kts
import ru.astrainteractive.klibs.rules.core.unit.model.UnitRole

plugins {
    alias(libs.plugins.klibs.rules.linter)
}

linter {
    unit(UnitRole.Impl()) // detected from the path when not set
    rootPackage.set("com.example.feature.impl") // detected from the sources when not set
}
```

<details>
<summary><b>Every option</b></summary>

```kotlin
import ru.astrainteractive.klibs.rules.core.unit.model.Checks
import ru.astrainteractive.klibs.rules.core.unit.model.ImplementationAccess
import ru.astrainteractive.klibs.rules.core.unit.model.RuleId
import ru.astrainteractive.klibs.rules.core.unit.model.UiComposition
import ru.astrainteractive.klibs.rules.core.unit.model.UnitRole
import ru.astrainteractive.klibs.rules.gradle.linter.util.toUnitReference

linter {
    unit(
        UnitRole.ImplUi(
            composition = UiComposition.HOST, // may depend on other features' UI units
            implementationAccess = ImplementationAccess.SIBLINGS, // may use the impl unit next to it
            checks = Checks(
                disabled = mapOf(RuleId.UNIT_README to "README migration is tracked in #123"),
                allowedDependencies = mapOf(projects.foo.toUnitReference() to "Reviewed integration boundary")
            )
        )
    )
    rootPackage.set("com.example.feature.ui")
    entryPointWords.addAll("slot", "panel") // your own entry-point kinds
    technologyWords.add("mongo") // your own technology packages
    pluralConcepts.add("goods") // plural package names that are fine
    platformClasses.add("WatchFaceService") // classes the platform constructs
    extensionKinds.add("slot") // kinds whose parts are extensions: fun SlotContext.backSlot()
    roleKinds.addAll("codec", "parser") // role kinds, named by the last word of their classes: PacketCodec
}
```

</details>

<details>
<summary><b>Roles</b></summary>

| `UnitRole`        | The unit                                              | Detected when not set                                 |
|-------------------|-------------------------------------------------------|-------------------------------------------------------|
| `Api`             | platform-free code other units call                   | path ends with `api`                                  |
| `Impl`            | an app's implementation of a feature                  | path ends with `impl`                                 |
| `ImplUi`          | an app's UI of a feature                              | -                                                     |
| `Context`         | an unsplit sub-context (`command`, `event-bukkit`)    | anything else                                         |
| `Platform`        | a feature's platform implementation (`gui/bukkit`)    | a platform name outside `core` and entry points       |
| `Technology`      | one technology's implementation (`data/exposed`)      | a known or dependency-derived technology name         |
| `EntryPoint`      | the platform main class and `RootModule`              | `application` plugin or a path under `:instances`     |
| `Contract`        | wire types a server and its client share              | path ends with `contract`                             |
| `StorageContract` | storage contracts (`data/dao`)                        | path ends with `dao`                                  |
| `Fake`            | shared test fakes                                     | path ends with `fake`                                 |
| `Core`            | an unsplit app core; no project dependencies          | -                                                     |
| `CoreApi`         | shared platform-free code                             | other platform-free `core` units                      |
| `CorePlatform`    | shared platform code                                  | the platform unit of `core`                           |
| `CoreUi`          | the design system                                     | `core/ui` and below                                   |

</details>

<details>
<summary><b>Tasks</b></summary>

| Task                             | Runs                                                                                           |
|----------------------------------|------------------------------------------------------------------------------------------------|
| `architectureLint`               | everything below and every detekt task, type resolution included                               |
| `architectureCheck`              | Konsist and structure checks of the unit; part of `check`                                      |
| `architectureGraphCheck`         | root: cycles in the unit graph, dependencies on an entry point; part of root `check`           |
| `architectureApiOwnershipReport` | root: API used by only one implementation unit, to `build/reports/architecture/api-ownership.txt` |
| `architectureBaseline`           | writes findings that cannot carry `@Suppress` to `architecture-baseline.txt`                   |
| `detektFormat`                   | detekt with auto-correct                                                                       |

</details>

```shell
./gradlew architectureLint --continue --no-configure-on-demand
```

`check` runs `detekt` without type resolution, so run `architectureLint` to see every finding.

Suppress a finding of any rule below with `@Suppress("RuleName")` on a declaration or `@file:Suppress("RuleName")`
on a file.

### 🔍 Checks

Rules marked *types* need type resolution and report only in `architectureLint`.

#### detekt

<details>
<summary><b>Naming</b></summary>

| Rule                        | Reports                                                                          |
|-----------------------------|----------------------------------------------------------------------------------|
| `ImplicitItParameter`       | the implicit lambda parameter `it`                                               |
| `ExplicitItName`            | a parameter, property, destructured entry or label named `it`                    |
| `FunctionVerb`              | a function starting with `get`, `map`, `make`, `do`, `handle` or `process`       |
| `ImplementationName`        | `Impl` and `Interface` suffixes, the `I` prefix on an interface                  |
| `RolelessClassName`         | `…Validator`, `…Factory`; `…WarmUp`, `…Migration`, `…ViewModel` outside their package |
| `KindNameInPackage`         | `Helper`, `Util`, `Manager`; `Model`/`Entity` without a technology word; a platform service outside `service` (*types*) |
| `BareIdentifier`            | an id typed `String`, `Int`, `Long` or `UUID`; a `…Id` that is not a value class |
| `TestFunctionName`          | a `@Test` not named `GIVEN_<state>_WHEN_<action>_THEN_<outcome>`                 |
| `CallbackDeclaration`       | an `on…` function-type parameter outside Compose, `…Listener` and `…Callback` interfaces |
| `CompanionFactoryFunction`  | a `create…` function in a companion object                                       |
| `PropertyAsFunction`        | a function without parameters that only returns a property                       |

</details>

<details>
<summary><b>Declarations and files</b></summary>

| Rule                        | Reports                                                                          |
|-----------------------------|----------------------------------------------------------------------------------|
| `DeclarationOrder`          | members not in the order properties, constructors, functions, `init`, companion  |
| `CalleeBeforeCaller`        | a function that calls one declared below it                                      |
| `OneTypeFamilyPerFile`      | a top-level function beside a type, two classes with behaviour in one file       |
| `ConstantOutsideCompanion`  | a constant in a class body outside its companion                                 |
| `NamespaceObject`           | an `object` that only groups functions                                           |
| `LocalOrInnerClass`         | a class inside a function, an `inner class`                                      |
| `DataClassDefaultValue`     | a default value in a data class that is not `@Serializable`                      |
| `AnonymousTuple`            | `Pair` and `Triple` outside `mapOf(a to b)`                                      |
| `ActualFileSuffix`          | an `actual` in a file without a platform suffix (`Name.android.kt`)              |
| `KDocHtmlTag`               | Javadoc HTML (`<p>`, `<code>`) in KDoc                                           |
| `KDocFiller`                | filler phrases, `@param` and `@return` that repeat a name, a summary over four sentences |

</details>

<details>
<summary><b>Formatting</b></summary>

| Rule                                   | Reports                                                               |
|----------------------------------------|-----------------------------------------------------------------------|
| `ExpressionBodyOnOneLine`              | an expression body that spans several lines                           |
| `PropertyInitializerOnDeclarationLine` | an initializer that starts on the line after its `=`                  |
| `ArgumentPerLine`                      | a call with three or more arguments not one per line                  |
| `BooleanOperatorWrapping`              | a line break around `&&` or `\|\|`                                    |
| `GuardIfBraces`                        | `if (…) { return }` that fits one line without braces                 |
| `LambdaSingleIfGuard`                  | a lambda whose whole body is one `if` without `else`                  |
| `OperatorBeforeTrailingCall`           | `(a * b).sp`, a chain of `plus`/`times` calls                         |
| `ExplicitInvoke`                       | `callback(…)` instead of `callback.invoke(…)`                         |
| `TrivialLambdaReference`               | a lambda that only forwards to a call instead of a reference (*types*) |

</details>

<details>
<summary><b>Compose</b></summary>

| Rule                            | Reports                                                                   |
|---------------------------------|---------------------------------------------------------------------------|
| `ComposableName`                | a UI composable not named `…Composable`                                   |
| `ComposableFileType`            | a class beside a top-level `@Composable`                                  |
| `ComposableWhenBranch`          | a multi-line `when` branch in a composable that is not a block            |
| `NonComposableFileInComposable` | a file in `composable` without a `@Composable`                            |
| `PreviewBlockLast`              | code after the first `@Preview`, a preview not named `…Preview`           |
| `RememberInName`                | a `remember…` function that calls no `remember`                           |
| `ComposeConstantCase`           | a constant in a composable or `theme` file not in its case                |
| `ThemeObjectFunction`           | a function inside a `theme` object                                        |

</details>

<details>
<summary><b>Coroutines and flows</b></summary>

| Rule                         | Reports                                                                        |
|------------------------------|--------------------------------------------------------------------------------|
| `HardcodedDispatcher`        | `Dispatchers.IO`/`Default`/`Unconfined` outside `di` and tests (*types*)       |
| `RunBlockingOutsideBoundary` | `runBlocking` outside `main` and technology packages                           |
| `RunBlockingInTest`          | `runBlocking` in tests                                                         |
| `RunTestResult`              | a `@Test` that does not return `runTest { … }` as `TestResult`                 |
| `TryCatchCancellation`       | a `catch` in suspend code that swallows `CancellationException`                |
| `RunCatchingCancellation`    | a suspend `runCatching` without `.propagateCancellationException()`            |
| `MutableStateFlowEmit`       | `emit`/`tryEmit` on a `MutableStateFlow`                                       |
| `ImpureUpdateLambda`         | an assignment to an outer name inside `update { }`                             |
| `StateFlowTimeout`           | `StateFlow.first()` inside `withTimeout`                                       |
| `SharedFlowEvent`            | `tryEmit` into a `MutableSharedFlow` without `extraBufferCapacity`             |
| `CallbackFlowShape`          | a `callbackFlow` without `awaitClose`, `launch { send }` in its callbacks      |
| `SingleJobLaunchMode`        | a launch on a single-job scope without an explicit mode                        |
| `BridgeCleanup`              | a bridge cleanup that writes external state or uses its own cancelled scope (*types*) |
| `DurationOverloadCall`       | `.inWholeMilliseconds` passed where a `Duration` overload exists               |
| `ThreadPrimitive`            | `Thread` created or used in main sources                                       |

</details>

<details>
<summary><b>Errors and output</b></summary>

| Rule                     | Reports                                                                            |
|--------------------------|------------------------------------------------------------------------------------|
| `ThrowInsideRunCatching` | `throw` inside `runCatching`                                                       |
| `ResultConsumption`      | a `Result` dropped without consuming it (*types*)                                  |
| `GetOrThrowInMain`       | `getOrThrow()` outside tests                                                       |
| `ConsoleOutput`          | `print`, `println`, `System.out`, `System.err`, `printStackTrace()`, `Thread.dumpStack()` |

</details>

<details>
<summary><b>DI and time</b></summary>

| Rule                            | Reports                                                                     |
|---------------------------------|-----------------------------------------------------------------------------|
| `DiModuleLifecycle`             | lifecycle misuse in DI modules that use the configured lifecycle (*types*)   |
| `RootModuleConstruction`        | a `RootModule` that constructs anything but DI modules and a `Lifecycle`    |
| `PropertyReadBeforeDeclaration` | a property read during construction before its declaration                 |
| `ClockSystemOutsideDi`          | `Clock.System` outside `di`, tests and previews                             |
| `SystemTimeCall`                | `System.currentTimeMillis()`, `System.nanoTime()`                           |

</details>

#### Konsist

The last column names the document section each check enforces.

<details>
<summary><b>Konsist checks</b></summary>

| Check | Reports | Enforces |
|---|---|---|
| `KindShape` | a file loose in the unit root or a concept package; a package below a kind package; a `composable` that holds both files and sub-packages, or a `model`, `util` or `theme` sub-package of it; in an app's `api` unit, a kind package other than `api`, `model`, `util`, `usecase`, `stub`, `di`, `component`, `viewmodel` (`memory` included), or anything but a state holder's interface in `component`/`viewmodel`; in a role kind, a top-level class whose name does not end with the kind's word (`…Limiter` in `limiter`), and a top-level function, interface or `object` | `arch-package.md`, Concepts and kinds 1; `arch-package.md`, Concepts and kinds 3 (role kind); `arch-package.md`, Units 5 |
| `RootPackagePath` | a root package that does not end with the unit name | `arch-package.md`, Units 3; `arch-package.kt.md`, Packages and directories |
| `KindImportDirection` | an import or qualified reference against the direction table: `model` imports only `model`; `util` imports `model`, `util`, `theme`, the data, enum and sealed types and the data objects nested in a contract and no other part of it, and `table`, `entity`, `database` in a technology unit; a role kind imports `model`, `util` and role kinds, and only use cases, implementations, entry-point kinds and `di` import it; a `fake` imports `api`, `model`, `util` and other fakes, and, whichever kind holds them, an interface of the unit that a class of its file implements (a DI-module interface in `di`) and the types its `override` members name; a screen imported by anything but what hosts or opens it; a domain model, contract or use case that imports a framework registration interface; `di` imported by anything but `di`, the entry point's root classes and a fake of a DI-module interface, which names that interface | `arch-context.md`, Direction and enforcement |
| `PackageDependencyCycle` | an import cycle between the packages of one compilation | `arch-context.md`, Direction and enforcement |
| `PlatformImport` | a platform or ORM import in `model`, `api`, `usecase` or an `Api` unit | `arch-domain.md`, What the model may import; `arch-package.md`, Contracts and implementations; `arch-context.md`, The dependency graph |
| `ContractPlacement` | an interface outside `api`, an implementation inside `api`, a `network`/`local`/`remote` package | `arch-package.md`, Contracts and implementations |
| `UseCaseShape` | a use case outside `usecase`, with several entries, taking another or passing through | `arch-domain.md`, Use case |
| `UseCaseAbstraction` | an interface or abstract class named `…UseCase` | `arch-package.md`, Contracts and implementations |
| `UtilShape` | a class, interface or `object` in `util`; a `util` function whose receiver, a parameter, a context parameter or a type-parameter bound is a contract of the project, an interface in an `api` package of the unit or of a dependency (`Repo?`, `List<Repo>`, a type alias of one and a qualified name included; an explicit import of a namesake hides a wildcard one), the data, enum and sealed types and data objects nested in a contract aside; an extension outside `util` | `arch-package.md`, Concepts and kinds 3 (`util`); `arch-package.md`, Files |
| `UtilFileName` | in `util`, a file of one receiver's extensions not named `<Receiver>Ext.kt`; anywhere, a file named `Utils.kt`, `Helpers.kt`, `Extensions.kt`, `Misc.kt` or the like | `arch-package.md`, Files; `arch-package.kt.md`, What goes in one file |
| `UtilFunctionAsExtension` | a `util` function whose form does not follow its signature, over the models of the unit and its dependencies, a typed identifier (`…Id`) not counted: a function that returns the type of a model parameter (`T`, `T?`, `Result<T>`), an extension of another model included (`JailInmate.cover(deposit: Money): Money`), or a top-level one whose model parameters are all of one type (`exceeds(amount: Money, limit: Money)`), is an extension of that type; an extension of a model that returns another type and takes a model of another type (`JailInmate.isBailCovered(deposit: Money)`) is a top-level function named after the decision. An extension whose model parameters are all of its receiver's type (`Money.exceeds(limit: Money)`), a top-level function over two or more model types that returns none of them, and `create<Type>` and `parse<Type>` are not reported | `arch-domain.md`, Rules live in `util` |
| `ForbiddenPackageName` | a forbidden package word; `table`, `entity`, `database` outside technology | `arch-package.md`, Concepts and kinds 10 |
| `PluralPackageName` | a plural package name, `news`, `settings` and `pluralConcepts` aside | `arch-package.md`, Concepts and kinds 9 |
| `PlatformPackage` | a package named after a platform in a multiplatform unit | `arch-package.kt.md`, Packages and directories |
| `ComposeKindWithoutCompose` | a `composable` or `theme` package in a unit without Compose | `arch-package.md`, Concepts and kinds 4 |
| `ThemeVisibility` | a `theme` object with the wrong visibility for its unit | `compose.kt.md` |
| `DiPlacement` | a `*Module` of the main sources outside `di`, the `fake` packages of a `<feature>/fake` unit aside: a `*Module` there, or in the test or `testFixtures` sources, is a fake of a DI-module interface | `arch-package.md`, Concepts and kinds 3 (`di`); `di-manual.kt.md` |
| `DiModuleVisibility` | a public DI module only its own unit constructs | `arch-package.kt.md`, Visibility; `di-manual.kt.md` |
| `RootModulePlacement` | a `RootModule` outside the entry point's `di` | `arch-package.md`, Units 1; `di-manual.kt.md` |
| `EntryPointShape` | an entry-point file outside its root and `di` | `arch-package.md`, Units 1 |
| `EntryPointVisibility` | a public class the platform invokes (`command`, `event`, `route`, …) | `arch-package.kt.md`, Visibility |
| `ImplementationVisibility` | a public class in a technology package | `arch-package.kt.md`, Visibility |
| `CoreShape` | a use case or a repository interface in a `core` unit | `arch-package.md`, Units 1 |
| `ContractUnitShape` | a type that is not `@Serializable` or logic in a `Contract` unit | `arch-package.kt.md`, Units and Gradle; `arch-context.md`, What crosses |
| `ErrorPlacement` | an error type outside `model` | `arch-package.md`, Files |
| `SealedFamilyFile` | a subtype of a sealed type declared in another file | `arch-package.md`, Files |
| `NestedDataClass` | a top-level data class only one class uses | `arch-package.md`, Files |
| `NestedEnum` | a top-level enum only one class uses | `arch-package.kt.md`, What goes in one file |
| `AnonymousProjectObject` | an anonymous `object :` implementing a project type | `arch-package.kt.md`, What goes in one file |
| `TestMirror` | a test in another package or source set than its subject | `arch-package.kt.md`, Tests |
| `FakePlacement` | a `Fake*` outside a `fake` package, `testFixtures` or a `Fake` unit; in a `Fake` unit, a class that is neither a `Fake*` nor an `InMemory*` in a `memory` package | `arch-package.kt.md`, Tests |
| `InMemoryPlacement` | an `InMemory*` outside a `memory` package of its source set: main, `test` or `testFixtures` | `arch-package.kt.md`, Tests |
| `AndroidNamespace` | an Android `namespace` that differs from the root package | `arch-package.kt.md`, Packages and directories |

</details>

#### Structure

<details>
<summary><b>Structure checks</b></summary>

| Check | Reports | Enforces |
|---|---|---|
| `UnitDependency` | a project dependency the unit's role may not have | `arch-context.md`, Direction and enforcement |
| `TechnologyDependency` | a dependency on a technology unit the unit's role may not have | `arch-context.md`, The dependency graph; `arch-context.md`, Direction and enforcement |
| `ApiUnitDependency` | an `Api` unit depending on anything but `Api`, `Core` or storage contracts | `arch-context.md`, Direction and enforcement |
| `EntryPointDependency` | a dependency on an entry-point unit | `arch-package.md`, Units 1; `arch-context.md`, The dependency graph |
| `EntryPointGraphDependency` | root: a dependency on an entry-point unit anywhere in the unit graph | `arch-package.md`, Units 1; `arch-context.md`, The dependency graph |
| `UnitDependencyCycle` | root: a cycle in the unit graph | `arch-context.md`, The dependency graph |
| `PluginImplUnit` | root: an `impl`, `impl-ui` or mixed `core` unit in a platform plugin | `arch-package.md`, Units 4; `arch-package.md`, Units 5 |
| `UnitReadme` | a unit without a one-sentence `README.md` | `arch-package.md`, Units 8 |
| `NestedUnit` | a unit inside another unit's directory | `arch-package.md`, Units 7 |
| `ContainerSources` | sources in a container directory above units (`modules/gui/src`) | `arch-package.md`, Units 7 |
| `UnitPath` | a unit at the build root instead of under a container such as `modules` | `arch-package.md`, Units 3 |
| `ProjectDirMatchesPath` | a unit directory that differs from its Gradle path | `arch-package.kt.md`, Units and Gradle |
| `PlatformSourceSetUnit` | a unit named after a Multiplatform target instead of a source set | `arch-package.md`, Units 5; `arch-package.kt.md`, Packages and directories |
| `RootSources` | sources in the root project | `arch-package.kt.md`, Units and Gradle |
| `RootBuildSubprojects` | `subprojects { }` or `allprojects { }` in the root build file | `arch-package.kt.md`, Units and Gradle |
| `ApiConfiguration` | an `api(…)` dependency not exported through a Multiplatform framework | `arch-package.kt.md`, Units and Gradle |
| `PublishedLibraryExplicitApi` | a published unit without strict explicit API | `arch-package.md`, Units 1; `arch-package.kt.md`, Visibility |
| `UnpublishedExplicitApi` | explicit API in a unit that is not published | `arch-package.kt.md`, Visibility |
| `ApiUnitPlatformDependency` | a platform dependency in an API unit outside a platform source set | `arch-package.kt.md`, Units and Gradle; `arch-context.md`, The dependency graph |
| `PlatformDependencyMetadata` | a platform dependency whose source set cannot be established | no document rule: the build metadata `ApiUnitPlatformDependency` needs |

</details>

### 📖 Details

<details>
<summary><b>Gradle plugins</b></summary>

- The linter plugin applies the detekt plugin and adds the architecture rules, the Konsist and structure checks
  and the tasks above. It uses the `detekt`, `konsist` and `structure` artifacts of its own version.
- detekt reads the bundled `detekt.yml` first, then `detekt-architecture.yml` and `detektRules.ruleConfigs`, then
  the project's own `config/detekt/detekt.yml` or `detekt { config }`. A later file wins.
- Type resolution uses the project's Kotlin JVM target, else the JVM that runs Gradle:
  `detektRules { jvmTarget.set("21") }` overrides it.
- `detekt` and `detektFormat` read every `.kt` and `.kts` of the project; `detektMain`, `detektTest` and the other
  source-set tasks read their source set with its classpath. Nested builds, worktrees, `build` and `resources`
  are skipped.
- The module's explicit API mode is passed to `detekt`.
- Configuration on demand is rejected because it can skip sibling projects; parallel execution and the
  configuration cache work.

</details>

<details>
<summary><b>Roles and exceptions</b></summary>

- `productKind` decides whether `impl` and `impl-ui` units are allowed. An `Api` unit exposing use-case
  interfaces needs it.
- `ImplUi.composition`: `LOCAL` (default) or `HOST`, which may also depend on other features' `ImplUi` units.
- `ImplUi.implementationAccess`: `SIBLINGS` (default) lets `:rating:ui` use `:rating:impl`; `NONE` forbids it. No
  setting allows another feature's `impl` or a cycle.
- `Platform`, `CorePlatform` and `Technology` units keep implementations directly in concept packages.
- `Checks` exceptions need a non-blank reason. An allowed dependency still counts for cycles. Disable
  `UNIT_DEPENDENCY_CYCLE` only at the root.
- Findings go to `build/reports/architecture/violations.txt`, accepted exceptions to
  `build/reports/architecture/policy.txt`. `Checks` affect only this linter's rules.
- `unitKind(UnitKind.API)` still works and gives the role's default policy. Repeating a declaration is fine;
  conflicting declarations fail the configuration.
- `maven-publish` marks a published library. A production dependency on a `Fake` unit is rejected even from an
  entry point.

</details>

<details>
<summary><b>What the linter knows about a unit</b></summary>

- detekt and Konsist get the same unit context: the role, the product kind, publication, technology words and
  each Kotlin compilation with its source directories and project dependencies.
- Compose is on when a compilation resolves a Compose dependency. An import, the compiler plugin alone or a
  test-only dependency does not turn it on.
- A platform dependency belongs to the source set that declares it: Android in `androidMain` is allowed in an app
  `Api`, `CoreApi` or `Core` unit, the same dependency in `commonMain` is not. A plugin's API stays platform-free.
  An origin that cannot be established is reported as missing metadata.
- Technology words come from the unit's libraries: the last segment of the group and the first word of the
  artifact (`org.mongodb:mongodb-driver-kotlin-coroutine` gives `mongodb`), also cut at a camel-case boundary
  (`VaultAPI` gives `vaultapi` and `vault`), generic words such as `core` and `runtime` aside. `technologyWords`
  adds to them.
- `RunCatchingCancellation` accepts the inline `CancellationException` check only when mikro-core is not in the
  version catalog.

</details>

<details>
<summary><b>How the checks read a project</b></summary>

- The checks cover what a rule states as a fixed name, number or shape: the package names, the concept/kind shape,
  the naming of implementations, the dependencies between units, and the platform and framework imports of
  `model`, `api` and `usecase` (the built-in framework list and `technologyWords`). Everything else in the rules is
  left to review; the Kotlin section of `TODO.md` lists what the rules state and no check reads.
- A kind is known by its package word: `model`, `api`, `usecase`, `util`, `di`, `theme`, `fake`; `stub`, `table`,
  `entity` and the technology names (`cache4k`, `datastore`, `exposed`, `file`, `krate`, `ktor`, `memory`,
  `okhttp`, `retrofit`, `room`, `sqldelight`) as implementation packages; the entry-point and UI words `command`,
  `component`, `composable`, `context`, `error`, `event`, `expansion`, `gui`, `job`, `menu`, `migration`, `route`,
  `screen`, `service`, `slot`, `viewmodel`, `warmup`. `entryPointWords`, `technologyWords` with the words of the
  unit's libraries, and `roleKinds` add to them. Any other package is a concept, and a concept that holds files is
  reported, except in the units that keep implementations in concept packages and at the entry point's root.
- A `roleKinds` word is a configuration error when it is not lower-case and trimmed, or when it is a fixed kind word
  or `database`, a technology word, an `entryPointWords` or `extensionKinds` word, one of `controller`, `helper`,
  `manager`, `utils`, a forbidden package name (`factory`, `dto`, …), or `module` (a `…Module` belongs to `di`): a
  role kind is named by a word of its own. The Konsist checks read the word, so the error fails `architectureCheck`
  of each unit with a main source directory; the root project and a unit without one run no Konsist check and do
  not report it.
- A unit's role is the one `unit(…)` declares, in its build file or in a convention plugin that configures
  `linter`; without it, the role is detected from the path (Roles). The checks apply the resolved role, so a declared
  role overrides the detected one and an `Api` unit may have a name other than `api`.
- A framework registration interface is recognised by its signature, never by its name: it sits in an entry-point
  kind of a unit that is not platform-free (`Api`, `CoreApi`, `StorageContract`, `Contract`), has no supertype and
  no property, and each of its functions is abstract, not `suspend`, returns nothing and takes only types of the
  built-in framework list (`org.bukkit.`, `io.ktor.`, `android.`, …). Whether the interface is responsible for
  registration, and a parameter type that does not resolve, are left to review.
- The direction checks do not establish whether a declaration needs to cross a unit boundary;
  `arch-package.md`, Units 6 decides that, and `architectureApiOwnershipReport` only gathers evidence for it.
- `architectureApiOwnershipReport` counts the references in a declaration's code (calls, supertypes and signatures,
  aliases included), not imports and not declared Gradle dependencies. Only production sources take part: tests
  and `testFixtures` are not consumers, and a reference from a `di` package is wiring, not shared behaviour. A
  single sibling consumer (`CANDIDATE`) is a reason to review ownership, not evidence to move the type; `UNKNOWN`
  does not prove a declaration unused; a published API (`EXTERNAL`) has consumers outside this build. Neither a
  candidate nor the number of candidates proves an API unnecessary or blocks the build.

</details>

<details>
<summary><b>Graph and API ownership</b></summary>

- Package cycles are checked per production or test compilation; unit cycles per production backend and
  variant. Test dependencies never become production edges.
- `architectureApiOwnershipReport` lists API declarations whose consumers reduce to one sibling implementation
  unit. It never fails the build and never moves code. Apply the linter to the root and every unit first.

| Status      | Means                                                                  |
|-------------|------------------------------------------------------------------------|
| `CANDIDATE` | one sibling owner found: review whether it belongs there               |
| `RETAIN`    | no sole owner                                                          |
| `UNKNOWN`   | no consumer outside DI                                                 |
| `UNCERTAIN` | ambiguous references, several owners or a feature DI reference         |
| `EXTERNAL`  | a published API                                                        |

The report reads sources lexically: overloads and shadowing can keep a declaration, and generated code, Java,
included builds and reflection are not seen. An empty list does not prove the layout is right.

</details>

<details>
<summary><b>Rule settings</b></summary>

```yaml
architecture:
  KindNameInPackage:
    serviceBaseTypes: [ 'example.framework.BackgroundWorker' ] # added to android.app.Service and others
    serviceAnnotations: [ 'example.framework.ManagedService' ]
  DiModuleLifecycle:
    lifecycleTypes: [ 'example.lifecycle.Lifecycle' ] # replaces the AstraLibs Lifecycle types
```

`detekt-architecture.yml` also configures detekt's own rules:

- `ClassOrdering` off, replaced by `DeclarationOrder`; `PrintStackTrace` off, replaced by `ConsoleOutput`.
- `ForbiddenImport`: `kotlinx.datetime.Instant`/`Clock`, `java.util.Date`; `java.time` and `Calendar` outside
  technology packages; DI frameworks (Koin, Dagger, Anvil, Metro, kotlin-inject, Kodein); `suspendCoroutine`,
  `trySendBlocking`, `Timer`, `Handler`, `thread`; `Executor` outside `core`; `CompletableFuture` outside technology
  packages; `painterResource` and `stringResource` outside `util`.
- `DataClassContainsFunctions` and `DataClassShouldBeImmutable`: a data class holds fields only.
- `ForbiddenComment` for region markers and separators, `WildcardImport` with no exceptions, ktlint
  `ImportOrdering`, `RedundantVisibilityModifier`, `GlobalCoroutineUsage`, lowercase `PackageNaming`,
  `SuspendFunSwallowedCancellation`, `SuspendFunInFinallySection`.
- `MagicNumber` accepts a number in a property declaration.

</details>

<details>
<summary><b>Without the Gradle plugin</b></summary>

**detekt.** Copy [`detekt-architecture.yml`](rules-kotlin/detekt/src/main/resources/detekt-architecture.yml) into
the project and list it after your own config:

```kotlin
dependencies {
    detektPlugins("ru.astrainteractive.klibs.rules:detekt:<version>")
    detektPlugins("dev.detekt:detekt-rules-ktlint-wrapper:<detekt-version>") // for ImportOrdering
}

detekt {
    buildUponDefaultConfig.set(true) // the rules switch themselves on in the jar's config.yml
    config.from(files("config/detekt/detekt.yml", "config/detekt/detekt-architecture.yml"))
}

// in a unit with explicitApi(), for RedundantVisibilityModifier
tasks.withType<dev.detekt.gradle.Detekt>().configureEach {
    freeCompilerArgs.add("-Xexplicit-api=strict")
}
```

**Konsist.** The checks run in their own JVM. The CLI checks one unit, writes one finding per line
(`rule<TAB>path<TAB>subject<TAB>message`) and always exits normally:

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

| Option           | Value                                                                       |
|------------------|-----------------------------------------------------------------------------|
| `--project-dir`  | the unit's directory; report paths are relative to it                       |
| `--unit-kind`    | the role in kebab case: `api`, `impl`, `impl-ui`, `core-api`, `context`, …  |
| `--unit-path`    | the unit's Gradle path, used to find the root package                       |
| `--unit-context` | the encoded context the Gradle plugin generates; needed by compilation-aware rules |
| `--main`         | a main source directory, optionally `sourceSet=path`; repeatable            |
| `--test`         | a test source directory, optionally `sourceSet=path`; repeatable            |
| `--report`       | the file findings are written to                                            |
| `--root-package` | the root package, when the unit path does not give it                       |

**Structure.** A plain library for build logic such as a convention plugin:

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

</details>

## Rust

`klibs-rules` checks a Cargo workspace: a required rustc, clippy and rustfmt configuration plus rules of its own.
The full reference (every key, role, rule and exception) is [`rules-rust/README.md`](rules-rust/README.md).

### 🚀 Installation

```shell
cargo install klibs-rules --version <latest-version> --locked
```

This installs the `cargo-klibs-rules` binary, which Cargo runs as `cargo klibs-rules`. The project gets no new
dependency and no build script.

### 🛠️ Configuration

```toml
# root Cargo.toml
[workspace]
resolver = "3"

[workspace.metadata.klibs-rules]
product-kind = "app" # or "platform-plugin"
```

```toml
# a member's Cargo.toml
[lints]
workspace = true

[package.metadata.klibs-rules]
role = "impl-ui" # detected from the path when not set; `core` and `impl-ui` are always declared
```

<details>
<summary><b>Required lints and toolchain</b></summary>

- Copy the `[workspace.lints]` block of [`rules-rust/Cargo.toml`](rules-rust/Cargo.toml) and
  [`rules-rust/clippy.toml`](rules-rust/clippy.toml) into the workspace root. The `LintConfig` rule reports every
  entry that is missing or lowered; extra lints are allowed.
- Pin the toolchain in `rust-toolchain.toml`, 1.93.0 or newer: the set of stable rustfmt keys differs by release,
  so an unpinned project can get a finding on one machine and none on another.
- A workspace with host frameworks names them in `platform-words`. Every other key is in
  [Configuration](rules-rust/README.md#configuration).

</details>

<details>
<summary><b>Roles</b></summary>

| Role               | Detected when not declared                                        |
|--------------------|-------------------------------------------------------------------|
| `tooling`          | the package is named `xtask`                                      |
| `proc-macro`       | its lib target is a `proc-macro`                                  |
| `entry-point`      | under `instances/`, or a crate without a Rust library target      |
| `core-ui`          | `modules/core/ui` or below it                                     |
| `core-platform`    | `modules/core/<word>` named in `platform-words`                   |
| `core-api`         | another path below `modules/core`                                 |
| `core`             | declaration only                                                  |
| `contract`         | final directory segment `contract`                                |
| `storage-contract` | final directory segment `dao`                                     |
| `api`              | final directory segment `api`                                     |
| `impl`             | final directory segment `impl`                                    |
| `impl-ui`          | declaration only                                                  |
| `platform`         | final directory segment named in `platform-words`                 |
| `technology`       | final directory segment matches a technology word of a dependency |
| `context`          | remaining crates                                                  |

The first matching row wins; the details are in [Roles](rules-rust/README.md#roles).

</details>

### ▶️ Run

```shell
cargo klibs-rules                                # the workspace in the current directory
cargo klibs-rules --manifest-path <Cargo.toml>   # another workspace
```

The command prints one line per finding (`<RuleId>`, path and line, subject, message) and writes
`violations.txt`, `policy.txt` and `api-ownership.txt` to `<target>/klibs-rules/`.

| Exit code | Means                                                  |
|-----------|--------------------------------------------------------|
| `0`       | the workspace is clean                                 |
| `1`       | there are findings                                     |
| `2`       | a configuration, metadata, write or argument error     |

<details>
<summary><b>In CI</b></summary>

```shell
cargo install klibs-rules --version <latest-version> --locked
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo klibs-rules
```

</details>

### 🔍 Checks

The rules cover the workspace layout and crate roles, the dependencies between crates, the module tree and kind
packages, naming, declarations, async code, error handling, DI, time and rustdoc. A finding is fixed in the code;
an exception is a `checks.disabled` or `checks.allowed-dependencies` entry with a reason in the manifest, or
`#[cfg_attr(klibs_rules, expect(<RuleId>, reason = "..."))]` in the source.

- [Rules](rules-rust/README.md#rules): every rule id and what it reports.
- [Exceptions](rules-rust/README.md#exceptions): when an exception is allowed and how it is written.
- [Report](rules-rust/README.md#report): the format of a finding.

## 🗂️ Development

| Path                         | Holds                                                           |
|------------------------------|-----------------------------------------------------------------|
| `rules-kotlin/detekt`        | the detekt rules, `config/config.yml`, `detekt-architecture.yml` |
| `rules-kotlin/gradle-plugin` | the detekt and linter Gradle plugins                            |
| `rules-kotlin/konsist`       | the Konsist checks and their CLI                                |
| `rules-kotlin/structure`     | the structure checks                                            |
| `rules-kotlin/self-check`    | the build that runs the real linter on this repository          |
| `rules-rust`                 | the Rust linter: the `klibs-rules` crate in `modules/rules`, its gate in the crate's tests |

```shell
cd rules-kotlin
./gradlew check                # tests, 100% line coverage, detekt and the architecture self-check
./gradlew publishToMavenLocal  # all artifacts to ~/.m2
```

```shell
cd rules-rust
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked  # KLIBS_RULES_BLESS=1 rewrites the expected files of the cases
```

<details>
<summary><b>Pull requests and releases</b></summary>

- A pull request runs the checks of the project it changes: `rules-kotlin/` or `rules-rust/`.
- A pull request that changes a project also changes its version: `klibs.project.version.string` in
  `rules-kotlin/gradle.properties`, `version` in `rules-rust/modules/rules/Cargo.toml`.
- A push to `master` releases the changed project: `rules-kotlin` to Maven Central with the tag
  `rules-kotlin-<version>`, `rules-rust` to crates.io with the tag `rules-rust-<version>`.

</details>

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
