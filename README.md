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
- **Multiplatform**: the detekt rules read syntax only and the Konsist checks parse source directories, so both
  work on `commonMain` as well as on JVM sources.

## 🚀 Installation

Three artifacts, one version:

| Artifact                                    | Use it for                                     |
|---------------------------------------------|------------------------------------------------|
| `ru.astrainteractive.klibs.rules:detekt`    | detekt plugin with the rules and their settings |
| `ru.astrainteractive.klibs.rules:konsist`   | Konsist checks, run from the command line      |
| `ru.astrainteractive.klibs.rules:structure` | structure checks, called from your build logic |

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

val konsistCheck = tasks.register<JavaExec>("konsistCheck") {
    val reportFile = konsistReport.get().asFile
    classpath = konsist
    mainClass.set("ru.astrainteractive.klibs.rules.konsist.cli.KonsistCli")
    args(
        "--project-dir", projectDir.absolutePath,
        "--unit-kind", "context",
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
| `--unit-kind`    | `context`, `library`, `api`, `contract` or `entry-point`, see below                |
| `--main`         | a main source directory; repeat it for every source set (`src/commonMain/kotlin`)  |
| `--test`         | a test source directory; repeatable                                                |
| `--report`       | the file the violations are written to                                             |
| `--root-package` | the unit's root package; detected from the sources when left out                   |

The unit kind decides which layout rules apply:

| Kind          | The unit                                                               |
|---------------|------------------------------------------------------------------------|
| `context`     | is a bounded context: use cases at the root, roles below it            |
| `library`     | is a published library or `core`: capability packages at the root      |
| `api`         | publishes a context's `<Context>Api` for other contexts                |
| `contract`    | holds the wire types a server and its client share                     |
| `entry-point` | holds the platform main class and the composition root                 |

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
| `NamespaceObject`              | an `object` that only groups functions                                           |
| `LocalOrInnerClass`            | a class inside a function body and an `inner class`                              |
| `TimeAsPrimitive`              | a time-named value (`createdAt`, `timeoutMs`) typed as a number or a string      |
| `KDocHtmlTag`                  | Javadoc HTML (`<p>`, `<code>`) in KDoc                                           |
| `ForbiddenPackageSegment`      | a layer or kind word in a package (`domain`, `usecase`, `dto`, ...)              |
| `ImplementationName`           | the `Impl` and `Interface` suffixes and the `I` prefix on an interface           |
| `KindNameInPackage`            | `Service` outside a `service` package, `Model`/`Entity` in a domain `model`      |

### detekt settings

`detekt-architecture.yml` configures detekt's own rules: `ClassOrdering`; `ForbiddenComment` for region
markers and separator comments; `ForbiddenImport` for `kotlinx.datetime.Instant`, and for platform and ORM
types in `model`; `RedundantVisibilityModifier`; `WildcardImport` with no exceptions; ktlint `ImportOrdering`.

### Konsist checks

| Check                     | Reports                                                                                     | Units        |
|---------------------------|---------------------------------------------------------------------------------------------|--------------|
| `RoleImportDirection`     | an import against the direction table: `model` depends on nothing, contracts on `model`, ... | context      |
| `PlatformImport`          | a platform or framework import in a use case or a contract                                  | context      |
| `LooseRootFile`           | anything at a context root but use cases, `<Context>Api` and `Default<Context>Api`          | context      |
| `UseCaseShape`            | a use case with more than one public entry, or one that takes another use case              | context      |
| `DefaultApiImplementsApi` | a `Default<Context>Api` that does not implement `<Context>Api` or is not at the root        | context      |
| `ContractLevel`           | a repository outside `data`, an implementation on the contract level, a `database` package  | context      |
| `EntryPointVisibility`    | a public command, listener, route or other entry point                                      | context      |
| `ErrorPlacement`          | an error type outside `model` and outside a technology package                              | context      |
| `DiPlacement`             | a DI module outside `di/`, a `di/` below a role, two DI modules in one package              | every unit   |
| `NestedDataClass`         | a top-level data class used as a property type by exactly one class                         | every unit   |
| `TestMirror`              | a test in another package than its owner                                                    | every unit   |
| `FakePlacement`           | a `Fake*` outside a `fake/` package                                                         | every unit   |
| `InMemoryPlacement`       | an `InMemory*` outside `data/memory` of the main sources                                    | every unit   |

### Structure checks

| Check            | Reports                                                                                      |
|------------------|----------------------------------------------------------------------------------------------|
| `UnitDependency` | an `api` unit depending on an `impl` unit, any other non-entry unit depending on an `impl`, a `contract` unit depending on a project unit |
| `UnitReadme`     | a unit without a `README.md` of one sentence                                                 |
| `NestedUnit`     | a unit inside the directory of another unit                                                  |
| `RootSources`    | sources in the root project                                                                  |

## 🗂️ Repository

| Path                     | Holds                                                                     |
|--------------------------|---------------------------------------------------------------------------|
| `rules-kotlin/detekt`    | the detekt rules, their `config/config.yml` and `detekt-architecture.yml` |
| `rules-kotlin/konsist`   | the Konsist checks and their command-line entry point                     |
| `rules-kotlin/structure` | the structure checks                                                      |

Build and test locally:

```shell
cd rules-kotlin
./gradlew check                # tests, 100% line coverage, detekt with these very rules
./gradlew publishToMavenLocal  # ru.astrainteractive.klibs.rules:{detekt,konsist,structure}
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