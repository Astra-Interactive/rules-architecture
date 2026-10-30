@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.gradle.linter.plugin.gradle

import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
import ru.astrainteractive.klibs.rules.gradle.fixture.IsolatedProjectsBuild
import java.io.File

class LinterPluginTest {
    private fun jailBuild(rootDirectory: File): IsolatedProjectsBuild {
        return IsolatedProjectsBuild(rootDirectory)
            .online()
            .withRootBuildScript("")
            .withModule(name = "jail", buildScript = JAVA_MODULE_SCRIPT)
            .withSourceFile("jail/README.md", "Jails players for a sentence and frees them when it ends.")
            .withSourceFile(
                "jail/src/main/kotlin/com/example/jail/jail/usecase/JailPlayerUseCase.kt",
                """
                package com.example.jail.jail.usecase

                class JailPlayerUseCase {
                    operator fun invoke() = Unit
                }
                """.trimIndent()
            )
            .withSourceFile(
                "jail/src/main/kotlin/com/example/jail/Jail.kt",
                """
                package com.example.jail

                data class Jail(val id: Int)
                """.trimIndent()
            )
            .withSourceFile(
                "jail/src/main/kotlin/com/example/jail/jail/model/Cell.kt",
                """
                package com.example.jail.jail.model

                class Cell
                """.trimIndent()
            )
    }

    @Test
    fun GIVEN_project_kind_word_WHEN_architecture_checked_THEN_accepts_its_package(@TempDir rootDirectory: File) {
        val moduleScript = JAVA_MODULE_SCRIPT + """

            linter {
                rootPackage.set("com.example.rating")
                entryPointWords.add("widget")
                technologyWords.add("mongo")
                pluralConcepts.add("goods")
            }
        """.trimIndent()
        val build = IsolatedProjectsBuild(rootDirectory)
            .online()
            .withRootBuildScript("")
            .withModule(name = "rating", buildScript = moduleScript)
            .withSourceFile("rating/README.md", "Shows the rating of players in widgets.")
            .withSourceFile(
                "rating/src/main/kotlin/com/example/rating/goods/widget/RatingWidget.kt",
                """
                package com.example.rating.goods.widget

                internal class RatingWidget
                """.trimIndent()
            )

        build.run(":rating:architectureCheck")
    }

    @Test
    fun GIVEN_technology_unit_copying_a_dependency_model_WHEN_architecture_checked_THEN_reports_the_copy(
        @TempDir rootDirectory: File
    ) {
        val exposedScript = """
            $JAVA_MODULE_SCRIPT

            dependencies { implementation(project(":dao")) }
        """.trimIndent()
        val build = IsolatedProjectsBuild(rootDirectory)
            .online()
            .withRootBuildScript("")
            .withModule(name = "dao", buildScript = JAVA_MODULE_SCRIPT)
            .withModule(name = "exposed", buildScript = exposedScript)
            .withSourceFile("dao/README.md", "Declares the rating storage contracts and models.")
            .withSourceFile("exposed/README.md", "Stores ratings in a database with Exposed.")
            .withSourceFile(
                "dao/src/main/kotlin/com/example/dao/rating/model/Rating.kt",
                """
                package com.example.dao.rating.model

                data class Rating(val name: String, val value: Int)
                """.trimIndent()
            )
            .withSourceFile(
                "exposed/src/main/kotlin/com/example/exposed/rating/RatingRow.kt",
                """
                package com.example.exposed.rating

                internal data class RatingRow(val name: String, val value: Int)
                """.trimIndent()
            )

        val result = build.runAndFail(":exposed:architectureCheck")

        assertTrue(result.output.contains("RatingRow copies the fields of Rating"), result.output)
    }

    @Test
    fun GIVEN_android_namespace_other_than_the_root_package_WHEN_architecture_checked_THEN_reports_it(
        @TempDir rootDirectory: File
    ) {
        val moduleScript = """
            $JAVA_MODULE_SCRIPT

            abstract class AndroidExtension {
                var namespace: String? = null
            }

            extensions.create<AndroidExtension>("android").namespace = "com.example.other"
        """.trimIndent()
        val build = IsolatedProjectsBuild(rootDirectory)
            .online()
            .withRootBuildScript("")
            .withModule(name = "rating", buildScript = moduleScript)
            .withSourceFile("rating/README.md", "Shows the rating of players.")
            .withSourceFile(
                "rating/src/main/kotlin/com/example/rating/rating/model/Rating.kt",
                """
                package com.example.rating.rating.model

                class Rating
                """.trimIndent()
            )

        val result = build.runAndFail(":rating:architectureCheck")

        assertTrue(
            result.output.contains("Set the namespace of this Android unit to its root package com.example.rating"),
            result.output
        )
    }

    @Test
    fun GIVEN_context_with_a_loose_root_file_WHEN_architecture_checked_THEN_fails_naming_it(
        @TempDir rootDirectory: File
    ) {
        val result = jailBuild(rootDirectory).runAndFail(":jail:check")

        assertTrue(
            result.output.contains("src/main/kotlin/com/example/jail/Jail.kt: Move Jail into a kind package"),
            "the loose root file was not reported:\n${result.output}"
        )
        assertTrue(result.output.contains("[KindShape]"), "the rule was not named:\n${result.output}")
        assertTrue(
            result.output.contains("run :jail:architectureBaseline to accept"),
            "the baseline task was not suggested:\n${result.output}"
        )
    }

    @Test
    fun GIVEN_baseline_of_current_violations_WHEN_architecture_checked_THEN_passes(@TempDir rootDirectory: File) {
        val build = jailBuild(rootDirectory)
        build.run(":jail:architectureBaseline")

        build.run(":jail:architectureCheck")

        val baseline = build.file("jail/architecture-baseline.txt").readLines()
        assertEquals(listOf("KindShape\tsrc/main/kotlin/com/example/jail/Jail.kt\tJail"), baseline)
    }

    @Test
    fun GIVEN_suppressed_violation_WHEN_architecture_checked_THEN_passes_without_baseline(
        @TempDir rootDirectory: File
    ) {
        val build = jailBuild(rootDirectory)
        val loose = build.file("jail/src/main/kotlin/com/example/jail/Jail.kt")
        val suppression = """@file:Suppress("KindShape")"""
        loose.writeText(suppression + System.lineSeparator().repeat(2) + loose.readText())

        build.run(":jail:architectureCheck")

        assertTrue(!build.file("jail/architecture-baseline.txt").exists(), "a baseline was created")
    }

    @Test
    fun GIVEN_violations_fixed_WHEN_baseline_written_again_THEN_removes_the_baseline(@TempDir rootDirectory: File) {
        val build = jailBuild(rootDirectory)
        build.run(":jail:architectureBaseline")
        val loose = build.file("jail/src/main/kotlin/com/example/jail/Jail.kt")
        val model = build.file("jail/src/main/kotlin/com/example/jail/jail/model/Jail.kt")
        model.writeText(loose.readText().replace("package com.example.jail", "package com.example.jail.jail.model"))
        loose.delete()

        build.run(":jail:architectureBaseline")

        assertTrue(!build.file("jail/architecture-baseline.txt").exists(), "an empty baseline was left behind")
    }

    @Test
    fun GIVEN_api_unit_depending_on_impl_unit_WHEN_architecture_checked_THEN_reports_the_dependency(
        @TempDir rootDirectory: File
    ) {
        val apiScript = """
            $JAVA_MODULE_SCRIPT

            dependencies {
                implementation(project(":impl"))
            }
        """.trimIndent()
        val build = IsolatedProjectsBuild(rootDirectory)
            .online()
            .withRootBuildScript("")
            .withModule(name = "api", buildScript = apiScript)
            .withModule(name = "impl", buildScript = JAVA_MODULE_SCRIPT)
            .withSourceFile("api/README.md", "Publishes what other contexts may call.")

        val result = build.runAndFail(":api:architectureCheck")

        val isDependencyReported = result.output.contains("An api unit never depends on an impl unit")
        val isRuleNamed = result.output.contains("[UnitDependency]")
        assertTrue(
            isDependencyReported && isRuleNamed,
            "the api to impl dependency was not reported:\n${result.output}"
        )
    }

    @Test
    fun GIVEN_unit_declaring_an_api_dependency_WHEN_architecture_checked_THEN_reports_the_configuration(
        @TempDir rootDirectory: File
    ) {
        val moduleScript = """
            plugins {
                `java-library`
                id("ru.astrainteractive.klibs.rules.gradle.linter")
            }

            dependencies {
                api("org.slf4j:slf4j-api:2.0.13")
                api(gradleApi())
            }
        """.trimIndent()
        val build = IsolatedProjectsBuild(rootDirectory)
            .online()
            .withRootBuildScript("")
            .withModule(name = "jail", buildScript = moduleScript)
            .withSourceFile("jail/README.md", "Jails players for a sentence and frees them when it ends.")

        val result = build.runAndFail(":jail:architectureCheck")

        val isDependencyReported = result.output.contains("Declare org.slf4j:slf4j-api with implementation")
        val isRuleNamed = result.output.contains("[ApiConfiguration]")
        assertTrue(
            isDependencyReported && isRuleNamed,
            "the api dependency was not reported:\n${result.output}"
        )
        assertTrue(!result.output.contains("null:unspecified"), result.output)
    }

    @Test
    fun GIVEN_kotlin_jvm_module_WHEN_compilation_and_architecture_run_THEN_ignores_automatic_stdlib(
        @TempDir rootDirectory: File
    ) {
        val moduleScript = """
            plugins {
                `java-library`
                id("org.jetbrains.kotlin.jvm")
                id("ru.astrainteractive.klibs.rules.gradle.linter")
            }

            afterEvaluate {
                configurations.getByName("compileClasspath").incoming.resolutionResult
            }
        """.trimIndent()
        val build = IsolatedProjectsBuild(rootDirectory)
            .online()
            .withRootBuildScript("")
            .withModule(name = "library", buildScript = moduleScript)
            .withSourceFile("library/README.md", "Publishes the library API.")

        build.run(":library:compileKotlin", ":library:architectureCheck")
        val cachedResult = build.run(":library:compileKotlin", ":library:architectureCheck")

        assertTrue(cachedResult.output.contains("Reusing configuration cache"), cachedResult.output)
    }

    @Test
    fun GIVEN_deferred_and_direct_dependencies_WHEN_baseline_written_THEN_checks_only_direct_declarations(
        @TempDir rootDirectory: File
    ) {
        val moduleScript = """
            $JAVA_MODULE_SCRIPT

            dependencies {
                api("org.apache.commons:commons-lang3:3.14.0")
            }
            val slf4jApi = dependencies.create("org.slf4j:slf4j-api:2.0.13")
            val paperApi = dependencies.create("io.papermc.paper:paper-api:1.21.4-R0.1-SNAPSHOT")
            val impl = dependencies.create(project(":impl"))
            configurations.getByName("api").withDependencies {
                add(slf4jApi)
            }
            configurations.getByName("implementation").withDependencies {
                add(paperApi)
                add(impl)
            }
        """.trimIndent()
        val build = IsolatedProjectsBuild(rootDirectory)
            .online()
            .withRootBuildScript("")
            .withModule(name = "api", buildScript = moduleScript)
            .withModule(name = "impl", buildScript = JAVA_MODULE_SCRIPT)
            .withSourceFile("api/README.md", "Publishes what other contexts may call.")

        build.run(":api:architectureBaseline")
        val cachedResult = build.run(":api:architectureBaseline")

        assertTrue(cachedResult.output.contains("Reusing configuration cache"), cachedResult.output)
        val baseline = build.file("api/architecture-baseline.txt").readLines()
        assertEquals(
            listOf("ApiConfiguration\tbuild.gradle.kts\tapi: org.apache.commons:commons-lang3"),
            baseline
        )
        build.run(":api:architectureCheck")
    }

    @Test
    fun GIVEN_kotlin_library_with_explicit_api_WHEN_configured_THEN_detekt_runs_the_architecture_rules(
        @TempDir rootDirectory: File
    ) {
        val moduleScript = """
            import dev.detekt.gradle.Detekt
            import ru.astrainteractive.klibs.rules.gradle.linter.extension.model.LinterExtension

            plugins {
                id("org.jetbrains.kotlin.jvm")
                `maven-publish`
                id("ru.astrainteractive.klibs.rules.gradle.linter")
            }

            kotlin {
                explicitApi()
            }

            val detektTask = tasks.named<Detekt>("detekt").get()
            val detektArguments = detektTask.freeCompilerArgs
            val detektConfig = detektTask.config
            val linterUnitKind = extensions.getByType<LinterExtension>().unitKind
            val detektPluginNames = configurations.getByName("detektPlugins").dependencies
                .map { dependency -> "${'$'}{dependency.group}:${'$'}{dependency.name}" }
            val linterReport = layout.buildDirectory.file("linter-report.txt")
            tasks.register("reportLinter") {
                val arguments = detektArguments
                val configFiles = detektConfig
                val unitKind = linterUnitKind
                val pluginNames = detektPluginNames
                val reportFile = linterReport
                outputs.file(reportFile)
                doLast {
                    val lines = listOf(
                        "args=" + arguments.get(),
                        "config=" + configFiles.files.map { file -> file.parentFile.parentFile.name + "/" + file.name },
                        "kind=" + unitKind.get(),
                        "plugins=" + pluginNames,
                    )
                    reportFile.get().asFile.writeText(lines.joinToString(separator = "\n"))
                }
            }
        """.trimIndent()
        val build = IsolatedProjectsBuild(rootDirectory)
            .withRootBuildScript("")
            .withModule(name = "library", buildScript = moduleScript)
            .withSourceFile("config/detekt/detekt.yml", PROJECT_IMPORT_LAYOUT)

        build.run(":library:reportLinter")

        val report = build.file("library/build/linter-report.txt").readLines()
        assertEquals("args=[-Xexplicit-api=strict]", report[0])
        assertEquals("config=[build/detekt.yml, build/detekt-architecture.yml, config/detekt.yml]", report[1])
        assertEquals("kind=library", report[2])
        assertTrue(report[3].contains("ru.astrainteractive.klibs.rules:detekt"), "rules missing: ${report[3]}")
    }

    @Test
    fun GIVEN_unknown_unit_kind_WHEN_architecture_checked_THEN_fails_listing_the_kinds(@TempDir rootDirectory: File) {
        val moduleScript = """
            $JAVA_MODULE_SCRIPT

            linter {
                unitKind.set("service")
            }
        """.trimIndent()
        val build = IsolatedProjectsBuild(rootDirectory)
            .online()
            .withRootBuildScript("")
            .withModule(name = "jail", buildScript = moduleScript)

        val result = build.runAndFail(":jail:architectureCheck")

        assertTrue(
            result.output.contains(
                "linter.unitKind of :jail is 'service', expected one of context, library, api, contract, entry-point"
            ),
            "the unknown kind was not reported:\n${result.output}"
        )
    }

    @Test
    fun GIVEN_context_depending_on_impl_only_in_tests_WHEN_architecture_checked_THEN_passes(
        @TempDir rootDirectory: File
    ) {
        val moduleScript = """
            $JAVA_MODULE_SCRIPT

            dependencies {
                testImplementation(project(":impl"))
            }
        """.trimIndent()
        val build = IsolatedProjectsBuild(rootDirectory)
            .online()
            .withRootBuildScript("")
            .withModule(name = "jail", buildScript = moduleScript)
            .withModule(name = "impl", buildScript = JAVA_MODULE_SCRIPT)
            .withSourceFile("jail/README.md", "Jails players for a sentence and frees them when it ends.")

        build.run(":jail:architectureCheck")
    }

    @Test
    fun GIVEN_fake_in_test_fixtures_outside_a_fake_package_WHEN_architecture_checked_THEN_judges_it_as_a_test(
        @TempDir rootDirectory: File
    ) {
        val build = jailBuild(rootDirectory)
            .withSourceFile(
                "jail/src/testFixtures/kotlin/com/example/jail/jail/FakeClock.kt",
                """
                package com.example.jail.jail

                object FakeClock
                """.trimIndent()
            )

        val result = build.runAndFail(":jail:architectureCheck")

        assertTrue(
            result.output.contains(
                "src/testFixtures/kotlin/com/example/jail/jail/FakeClock.kt: Move FakeClock to a fake package"
            ),
            "the testFixtures fake was not judged as a test source:\n${result.output}"
        )
    }

    @Test
    fun GIVEN_root_project_with_sources_WHEN_architecture_checked_THEN_reports_them(@TempDir rootDirectory: File) {
        val build = IsolatedProjectsBuild(rootDirectory)
            .online()
            .withRootBuildScript(JAVA_MODULE_SCRIPT)
            .withModule(name = "jail", buildScript = "")
            .withSourceFile("src/main/kotlin/com/example/Main.kt", "package com.example\n")

        val result = build.runAndFail(":architectureCheck")

        val isRootReported = result.output.contains("src: Move the sources of the root project into a unit")
        val isNestedReported = result.output.contains("[NestedUnit]")
        assertTrue(
            isRootReported && !isNestedReported,
            "the root sources were not reported, or units below the root were:\n${result.output}"
        )
    }

    @Test
    fun GIVEN_kotlin_unit_with_explicit_api_warning_WHEN_configured_THEN_detekt_gets_the_warning_mode(
        @TempDir rootDirectory: File
    ) {
        val moduleScript = """
            import dev.detekt.gradle.Detekt

            plugins {
                id("org.jetbrains.kotlin.jvm")
                id("ru.astrainteractive.klibs.rules.gradle.linter")
            }

            kotlin {
                explicitApiWarning()
            }

            val detektArguments = tasks.named<Detekt>("detekt").get().freeCompilerArgs
            val argumentsReport = layout.buildDirectory.file("detekt-arguments.txt")
            tasks.register("reportDetektArguments") {
                val arguments = detektArguments
                val reportFile = argumentsReport
                outputs.file(reportFile)
                doLast { reportFile.get().asFile.writeText(arguments.get().joinToString()) }
            }
        """.trimIndent()
        val build = IsolatedProjectsBuild(rootDirectory)
            .withRootBuildScript("")
            .withModule(name = "library", buildScript = moduleScript)

        build.run(":library:reportDetektArguments")

        assertEquals("-Xexplicit-api=warning", build.file("library/build/detekt-arguments.txt").readText())
    }

    @Test
    fun GIVEN_test_in_another_source_set_WHEN_architecture_checked_THEN_reports_the_expected_set(
        @TempDir rootDirectory: File
    ) {
        val build = IsolatedProjectsBuild(rootDirectory)
            .online()
            .withRootBuildScript("")
            .withModule(name = "jail", buildScript = JAVA_MODULE_SCRIPT)
            .withSourceFile("jail/README.md", "Jails players for a sentence and frees them when it ends.")
            .withSourceFile(
                "jail/src/commonMain/kotlin/com/example/jail/model/Jail.kt",
                "package com.example.jail.model\n\nclass Jail"
            )
            .withSourceFile(
                "jail/src/jvmTest/kotlin/com/example/jail/model/JailTest.kt",
                "package com.example.jail.model\n\nclass JailTest"
            )

        val result = build.runAndFail(":jail:architectureCheck")

        assertTrue(result.output.contains("Move JailTest to commonTest"), result.output)
    }

    @Test
    fun GIVEN_inherited_framework_export_WHEN_checked_THEN_only_unrelated_api_dependency_is_reported(
        @TempDir rootDirectory: File
    ) {
        val moduleScript = """
            plugins {
                id("org.jetbrains.kotlin.multiplatform")
                id("ru.astrainteractive.klibs.rules.gradle.linter")
            }

            configurations {
                val exportedApi by creating
                val unrelatedApi by creating
            }

            kotlin {
                iosX64()
                binaries {
                    framework {
                        configurations.getByName(exportConfigurationName)
                            .extendsFrom(configurations.getByName("exportedApi"))
                    }
                }
            }

            dependencies {
                add("exportedApi", "org.slf4j:slf4j-api:2.0.13")
                exportedApi(project(":core"))
                unrelatedApi("org.apache.commons:commons-lang3:3.14.0")
                unrelatedApi(project(":core"))
            }
        """.trimIndent()
        val build = IsolatedProjectsBuild(rootDirectory)
            .online()
            .withRootBuildScript("")
            .withModule(name = "library", buildScript = moduleScript)
            .withModule(name = "core", buildScript = JAVA_MODULE_SCRIPT)
            .withSourceFile("library/README.md", "Publishes the library API.")
            .withSourceFile("core/README.md", "Provides shared models.")

        val result = build.runAndFail(":library:architectureCheck")

        assertTrue(result.output.contains("org.apache.commons:commons-lang3"), result.output)
        assertTrue(!result.output.contains("Declare org.slf4j:slf4j-api"), result.output)
    }

    @Test
    fun GIVEN_configuration_named_export_without_framework_WHEN_checked_THEN_does_not_exempt_api(
        @TempDir rootDirectory: File
    ) {
        val moduleScript = """
            plugins {
                id("org.jetbrains.kotlin.multiplatform")
                id("ru.astrainteractive.klibs.rules.gradle.linter")
            }

            configurations {
                val exportedApi by creating
                val customExport by creating { extendsFrom(exportedApi) }
            }

            dependencies {
                add("exportedApi", "org.slf4j:slf4j-api:2.0.13")
            }
        """.trimIndent()
        val build = IsolatedProjectsBuild(rootDirectory)
            .online()
            .withRootBuildScript("")
            .withModule(name = "library", buildScript = moduleScript)
            .withSourceFile("library/README.md", "Publishes the library API.")

        val result = build.runAndFail(":library:architectureCheck")

        assertTrue(result.output.contains("Declare org.slf4j:slf4j-api"), result.output)
    }

    private companion object {
        val JAVA_MODULE_SCRIPT = """
            plugins {
                `java-library`
                id("ru.astrainteractive.klibs.rules.gradle.linter")
            }
        """.trimIndent()

        val PROJECT_IMPORT_LAYOUT = """
            ktlint:
              ImportOrdering:
                active: true
                layout: '*,java.**,javax.**,kotlin.**'
        """.trimIndent()
    }
}
