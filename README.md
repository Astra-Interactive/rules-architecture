[![version](https://img.shields.io/maven-central/v/ru.astrainteractive.klibs.rules/detekt?style=flat-square)](https://github.com/Astra-Interactive/rules-architecture)
[![kotlin_version](https://img.shields.io/badge/kotlin-2.4.10-blueviolet?style=flat-square)](https://github.com/Astra-Interactive/rules-architecture)
[![detekt_version](https://img.shields.io/badge/detekt-2.0.0--alpha.6-orange?style=flat-square)](https://detekt.dev)

## Rules Architecture

Rules Architecture turns the package-layout and architecture rules of our Kotlin projects into checks that
fail the build. A rule that no tool checks decays, so instead of trusting a document, the checks read your
sources and tell you which file breaks which rule and how to fix it.

## ✨ Features

- **Per-file rules for detekt**: 13 custom rules plus settings for detekt's own rules, with IDE support and
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

Four artifacts, one version:

| Artifact                                       | Use it for                                                      |
|------------------------------------------------|-----------------------------------------------------------------|
| `ru.astrainteractive.klibs.rules:gradle-plugin` | Gradle plugins that wire the three artifacts below into a build |
| `ru.astrainteractive.klibs.rules:detekt`       | detekt plugin with the rules and their settings                 |
| `ru.astrainteractive.klibs.rules:konsist`      | Konsist checks, run from the command line                       |
| `ru.astrainteractive.klibs.rules:structure`    | structure checks, called from your build logic                  |

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
`mavenCentral()` to the `pluginManagement` repositories, then apply a plugin to every unit:

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
  HTML reports, parallel execution and `allRules`.
- Resolves types against the Kotlin JVM target of the project, or the JVM that runs Gradle when there is none.
  Set another target with `detektRules { jvmTarget.set("21") }`.

**Linter plugin.** Apply it instead of the detekt plugin.

- Adds the architecture detekt rules and their `detekt-architecture.yml`, listed after `detekt.yml` so they win,
  and passes the module's explicit API mode to the `detekt` task.
- Registers `architectureCheck`, which `check` depends on: the Konsist checks of the unit's package layout, run in
  a separate JVM, and the structure checks of its README, nested units and project dependencies.
- Registers `architectureBaseline`, which writes the current violations to `architecture-baseline.txt`; the check
  accepts what the file lists, so old code can stay as it is.

The unit kind decides which layout rules apply. It is detected by convention and can be set:

```kotlin
linter {
    unitKind.set("library") // context, library, api, contract, entry-point or technology
    rootPackage.set("ru.astrainteractive.aspekt.module.jail")
}
```

| Detected as       | When                                                                                     |
|-------------------|------------------------------------------------------------------------------------------|
| `api`, `contract` | the last segment of the project path                                                     |
| `technology`      | a technology name as the last segment of the project path (`data/exposed`, `data/room`) |
| `entry-point`     | the `application` or `com.android.application` plugin, or a path under `:instances`      |
| `library`         | Kotlin explicit API mode strict                                                          |
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
| `--main`         | a main source directory; repeat it for every source set (`src/commonMain/kotlin`)  |
| `--test`         | a test source directory; repeatable                                                |
| `--report`       | the file the violations are written to                                             |
| `--root-package` | the unit's root package, when the unit path does not find it                       |

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
| `AnonymousTuple`               | `Pair` and `Triple` used as a type or constructed by name                        |
| `DataClassDefaultValue`        | a default value in a data class that is not `@Serializable`                      |
| `ConstructorDefaultDependency` | a constructor default that creates a dependency, such as `= Foo()`               |
| `TestFunctionName`             | a `@Test` not named `GIVEN_<state>_WHEN_<action>_THEN_<outcome>`                 |
| `CalleeBeforeCaller`           | a function that calls a function declared below it in the same class or file     |
| `DeclarationOrder`             | properties, secondary constructors, functions, `init` blocks, companion: out of this order |
| `NamespaceObject`              | an `object` that only groups functions                                           |
| `LocalOrInnerClass`            | a class inside a function body and an `inner class`                              |
| `TimeAsPrimitive`              | a time-named value (`createdAt`, `timeoutMs`) typed as a number or a string      |
| `KDocHtmlTag`                  | Javadoc HTML (`<p>`, `<code>`) in KDoc                                           |
| `ImplementationName`           | the `Impl` and `Interface` suffixes and the `I` prefix on an interface           |
| `KindNameInPackage`            | `Service` outside a `service` package, `Model`/`Entity` in a domain `model`, `Helper`/`Util`/`Manager` names |
| `ExpressionBodyOnOneLine`      | an expression body (`= …`) with a line break after the `=` or inside the body     |
| `BooleanOperatorWrapping` | a line break on either side of `&&` or `\|\|` |
| `ArgumentPerLine` | a call or constructor call with three or more arguments in its parentheses, not each on its own line |
| `ExplicitInvoke` | a call `name(…)` of a parameter, property or local value instead of `name.invoke(…)` |
| `FunctionVerb` | a function named `get…` (but `getState`) or with the word `map`, `make`, `do`, `handle` or `process` |
| `PropertyAsFunction` | a function with no parameters that only returns a property of its class or file |
| `OneTypeFamilyPerFile` | a top-level function (a composable included) beside a top-level type, and a second class with behaviour in one file |
| `ConstantOutsideCompanion` | a `const val` or literal `val` in a class body outside its companion, a `private object` of constants beside a type |
| `ComposableFileType` | a class, interface or object in a file with a `@Composable` function, except a `*Defaults` object |
| `ThemeObjectFunction` | a function inside an object of a `theme` package |
| `ComposeConstantCase` | a `const val` not `UPPER_SNAKE` or a `val` not `PascalCase` in a composable or `theme` file |
| `PreviewBlockLast` | a non-preview declaration after the first `@Preview`, a `@Preview` not named `…Preview` |
| `RememberInName` | a `remember…` function whose body calls no `remember` |
| `ActualFileSuffix` | a file with an `actual` declaration whose name has no platform suffix (`Name.android.kt`) |
| `MutableStateFlowEmit` | `emit`/`tryEmit` on a property declared or initialised as a `MutableStateFlow` |
| `ImpureUpdateLambda` | an assignment to an outer name inside the lambda of `update`, `getAndUpdate` or `updateAndGet` |
| `SingleJobLaunchMode` | a launch on a single-job scope without a `SingleJobMode`, an unguarded launch in an `on…` handler of a view model or component |
| `ThrowInsideRunCatching` | a `throw` inside the lambda of `runCatching`, `mapCatching` or `recoverCatching` |
| `RunCatchingCancellation` | a `runCatching` in a `suspend` function or coroutine lambda not followed by `.propagateCancellationException()` |
| `GetOrThrowInMain` | `getOrThrow()` outside the test sources |
| `BareIdentifier` | a property named `id`/`…Id` typed `String`, `Int`, `Long` or `UUID` outside a `@Serializable` class or technology package, a `…Id` class that is not a value class |
| `CompanionFactoryFunction` | a `create…`, `of` or `from` function in a companion object |
| `RolelessClassName` | a `…Validator` or `…Factory` class, `…WarmUp`/`…Migration`/`…ViewModel` outside its `warmup`/`migration`/`viewmodel` package |
| `DiModuleLifecycle` | a `…Module` that declares `enable`, `disable`, `reload`, `onEnable`, `onDisable` or `onReload` |
| `RootModuleConstruction` | a `RootModule` property that constructs anything but a `…Module` or a `Lifecycle` |
| `SystemTimeCall` | `System.currentTimeMillis()` and `System.nanoTime()` |
| `RunBlockingInTest` | `runBlocking` in a test source set |
| `KDocFiller` | a KDoc filler phrase, a `@return` of "the result", a `@param` that repeats its name, a summary of more than four sentences |

### detekt settings

`detekt-architecture.yml` configures detekt's own rules: `ClassOrdering` off, replaced by `DeclarationOrder`,
which puts `init` blocks after the functions they call; `ForbiddenComment` for region
markers and separator comments; `ForbiddenImport` for `kotlinx.datetime.Instant`, `kotlinx.datetime.Clock` and `java.util.Date`, for `java.time` types and `java.util.Calendar` outside technology packages (`ForbiddenImport/time`), and for platform and ORM
types in `model`; `RedundantVisibilityModifier`; `WildcardImport` with no exceptions; ktlint `ImportOrdering`.
`DataClassContainsFunctions`, with no conversion exception, and `DataClassShouldBeImmutable`: a data class
holds fields only. `GlobalCoroutineUsage`: no global scope. `PackageNaming` with a lowercase-only pattern.

### Konsist checks

| Check                  | Reports                                                                                                   |
|------------------------|-----------------------------------------------------------------------------------------------------------|
| `KindShape`            | a file loose in the unit root or a concept package, a package below a kind package                        |
| `KindImportDirection`  | an import against the first table of `arch-context.md`: `model` depends on nothing of its level, ...      |
| `PlatformImport`       | a platform or ORM import in `model`, `api` or `usecase`, or anywhere in an `api` unit                     |
| `UseCaseShape`         | `*UseCase` outside `usecase`, other names in `usecase`, more than one entry, a use case taking another    |
| `ContractPlacement`    | a `*Repository` outside `api`, an implementation inside `api`, a `network`/`local`/`remote` package       |
| `ForbiddenPackageName` | a forbidden word in a package created inside the unit; the root package may keep its unit's word          |
| `UtilShape`            | a class or object in `util`, an extension function outside `util` and outside its receiver's file          |
| `EntryPointVisibility` | a public class in a kind the platform invokes (`command`, `event`, `route`, `service`, `warmup`, ...)     |
| `ErrorPlacement`       | an error type outside a `model` package                                                                   |
| `DiPlacement`          | a `*Module` outside a `di` package                                                                        |
| `NestedDataClass`      | a top-level data class used as a property type by exactly one class                                       |
| `TestMirror`           | a test in another package than its owner                                                                  |
| `FakePlacement`        | a `Fake*` outside a `fake` package                                                                        |
| `InMemoryPlacement`    | an `InMemory*` outside a `memory` package of the main sources                                              |
| `SealedFamilyFile` | a class or object whose direct supertype is a sealed type declared in another file |
| `StateTypePlacement` | a `<Screen>State` nested in another type or declared outside `model` |
| `NestedEnum` | a top-level enum used as the property type of exactly one class |
| `UtilFileName` | a `util` file of one receiver's extensions not named `<Receiver>Ext.kt`, a file named `Utils.kt`, `Helpers.kt`, `Extensions.kt` and the like |
| `UtilFunctionAsExtension` | a `util` function whose one parameter is a model of the unit, instead of an extension |
| `ImplementationVisibility` | a public class in a technology package or a technology unit's concept, outside a library |
| `UseCaseAbstraction` | an interface or abstract class named `…UseCase` |
| `ContractName` | an `api` interface named `…Dao`, `…Storage` or `…Krate`, a `…Repository` extending a generic repository or `Crud` base |
| `CoreShape` | a `…UseCase` class or a `…Repository` interface in a `core` unit |
| `RootModulePlacement` | a `RootModule` outside the entry-point unit or its `di` package |
| `EntryPointShape` | a file of an entry-point unit outside its root and its `di` package |
| `ContractUnitShape` | in a `contract` unit, a class or interface that is not `@Serializable`, a function with a body outside an object |

### Structure checks

| Check            | Reports                                                                                      |
|------------------|----------------------------------------------------------------------------------------------|
| `UnitDependency` | a dependency on an `impl` unit or a feature's platform unit (`gui/bukkit`) outside the entry point, `core` depending on a feature, a `contract` unit depending on a project unit |
| `UnitReadme`     | a unit without a `README.md` of one sentence                                                 |
| `NestedUnit`     | a unit inside the directory of another unit                                                  |
| `RootSources`    | sources in the root project                                                                  |

## 🗂️ Repository

| Path                     | Holds                                                                     |
|--------------------------|---------------------------------------------------------------------------|
| `rules-kotlin/detekt`      | the detekt rules, their `config/config.yml` and `detekt-architecture.yml` |
| `rules-kotlin/gradle-plugin` | the detekt and linter Gradle plugins                                    |
| `rules-kotlin/konsist`     | the Konsist checks and their command-line entry point                     |
| `rules-kotlin/structure`   | the structure checks                                                      |

Build and test locally:

```shell
cd rules-kotlin
./gradlew check                # tests, 100% line coverage, detekt with these very rules
./gradlew publishToMavenLocal  # ru.astrainteractive.klibs.rules:{detekt,gradle-plugin,konsist,structure}
```

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