@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.gradle.linter.plugin.gradle

import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
import ru.astrainteractive.klibs.rules.gradle.fixture.IsolatedProjectsBuild
import java.io.File

// Keep the existing TestKit scenarios together; new policy scenarios have their own test class.
@Suppress("LargeClass")
class LinterPluginTest {
    private fun jailBuild(rootDirectory: File): IsolatedProjectsBuild {
        return IsolatedProjectsBuild(rootDirectory)
            .online()
            .withRootBuildScript("")
            .withModule(name = "modules:jail", buildScript = JAVA_MODULE_SCRIPT)
            .withSourceFile("modules/jail/README.md", "Jails players for a sentence and frees them when it ends.")
            .withSourceFile(
                "modules/jail/src/main/kotlin/com/example/jail/jail/usecase/JailPlayerUseCase.kt",
                """
                package com.example.jail.jail.usecase

                class JailPlayerUseCase {
                    operator fun invoke() = Unit
                }
                """.trimIndent()
            )
            .withSourceFile(
                "modules/jail/src/main/kotlin/com/example/jail/Jail.kt",
                """
                package com.example.jail

                data class Jail(val id: Int)
                """.trimIndent()
            )
            .withSourceFile(
                "modules/jail/src/main/kotlin/com/example/jail/jail/model/Cell.kt",
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
            .withModule(name = "modules:rating", buildScript = moduleScript)
            .withSourceFile("modules/rating/README.md", "Shows the rating of players in widgets.")
            .withSourceFile(
                "modules/rating/src/main/kotlin/com/example/rating/goods/widget/RatingWidget.kt",
                """
                package com.example.rating.goods.widget

                internal class RatingWidget
                """.trimIndent()
            )

        build.run(":modules:rating:architectureCheck")
    }

    @Test
    fun GIVEN_project_role_kind_WHEN_architecture_checked_THEN_accepts_its_package(@TempDir rootDirectory: File) {
        val moduleScript = JAVA_MODULE_SCRIPT + """

            linter {
                rootPackage.set("com.example.vote")
                roleKinds.add("limiter")
            }
        """.trimIndent()
        val build = IsolatedProjectsBuild(rootDirectory)
            .online()
            .withRootBuildScript("")
            .withModule(name = "modules:vote", buildScript = moduleScript)
            .withSourceFile("modules/vote/README.md", "Limits how often players vote.")
            .withSourceFile(
                "modules/vote/src/main/kotlin/com/example/vote/vote/limiter/VoteCooldownLimiter.kt",
                """
                package com.example.vote.vote.limiter

                internal class VoteCooldownLimiter
                """.trimIndent()
            )

        build.run(":modules:vote:architectureCheck")
    }

    @Test
    fun GIVEN_storage_representation_matching_a_model_WHEN_architecture_checked_THEN_leaves_the_choice_to_review(
        @TempDir rootDirectory: File
    ) {
        val exposedScript = """
            $JAVA_MODULE_SCRIPT

            dependencies { implementation(project(":modules:dao")) }
        """.trimIndent()
        val build = IsolatedProjectsBuild(rootDirectory)
            .online()
            .withRootBuildScript("")
            .withModule(name = "modules:dao", buildScript = JAVA_MODULE_SCRIPT)
            .withModule(name = "modules:exposed", buildScript = exposedScript)
            .withSourceFile("modules/dao/README.md", "Declares the rating storage contracts and models.")
            .withSourceFile("modules/exposed/README.md", "Stores ratings in a database with Exposed.")
            .withSourceFile(
                "modules/dao/src/main/kotlin/com/example/dao/rating/model/Rating.kt",
                """
                package com.example.dao.rating.model

                data class Rating(val name: String, val value: Int)
                """.trimIndent()
            )
            .withSourceFile(
                "modules/exposed/src/main/kotlin/com/example/exposed/rating/RatingRow.kt",
                """
                package com.example.exposed.rating

                internal data class RatingRow(val name: String, val value: Int)
                """.trimIndent()
            )

        build.run(":modules:exposed:architectureCheck")
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
            .withModule(name = "modules:rating", buildScript = moduleScript)
            .withSourceFile("modules/rating/README.md", "Shows the rating of players.")
            .withSourceFile(
                "modules/rating/src/main/kotlin/com/example/rating/rating/model/Rating.kt",
                """
                package com.example.rating.rating.model

                class Rating
                """.trimIndent()
            )

        val result = build.runAndFail(":modules:rating:architectureCheck")

        assertTrue(
            result.output.contains("Set the namespace of this Android unit to its root package com.example.rating"),
            result.output
        )
    }

    @Test
    fun GIVEN_context_with_a_loose_root_file_WHEN_architecture_checked_THEN_fails_naming_it(
        @TempDir rootDirectory: File
    ) {
        val result = jailBuild(rootDirectory).runAndFail(":modules:jail:check")

        assertTrue(
            result.output.contains("src/main/kotlin/com/example/jail/Jail.kt: Move Jail into a kind package"),
            "the loose root file was not reported:\n${result.output}"
        )
        assertTrue(result.output.contains("[KindShape]"), "the rule was not named:\n${result.output}")
        assertTrue(
            result.output.contains("run :modules:jail:architectureBaseline to accept"),
            "the baseline task was not suggested:\n${result.output}"
        )
    }

    @Test
    fun GIVEN_baseline_of_current_violations_WHEN_architecture_checked_THEN_passes(@TempDir rootDirectory: File) {
        val build = jailBuild(rootDirectory)
        build.run(":modules:jail:architectureBaseline")

        build.run(":modules:jail:architectureCheck")

        val baseline = build.file("modules/jail/architecture-baseline.txt").readLines()
        assertEquals(listOf("KindShape\tsrc/main/kotlin/com/example/jail/Jail.kt\tJail"), baseline)
    }

    @Test
    fun GIVEN_suppressed_violation_WHEN_architecture_checked_THEN_passes_without_baseline(
        @TempDir rootDirectory: File
    ) {
        val build = jailBuild(rootDirectory)
        val loose = build.file("modules/jail/src/main/kotlin/com/example/jail/Jail.kt")
        val suppression = """@file:Suppress("KindShape")"""
        loose.writeText(suppression + System.lineSeparator().repeat(2) + loose.readText())

        build.run(":modules:jail:architectureCheck")

        assertTrue(!build.file("modules/jail/architecture-baseline.txt").exists(), "a baseline was created")
    }

    @Test
    fun GIVEN_violations_fixed_WHEN_baseline_written_again_THEN_removes_the_baseline(@TempDir rootDirectory: File) {
        val build = jailBuild(rootDirectory)
        build.run(":modules:jail:architectureBaseline")
        val loose = build.file("modules/jail/src/main/kotlin/com/example/jail/Jail.kt")
        val model = build.file("modules/jail/src/main/kotlin/com/example/jail/jail/model/Jail.kt")
        model.writeText(loose.readText().replace("package com.example.jail", "package com.example.jail.jail.model"))
        loose.delete()

        build.run(":modules:jail:architectureBaseline")

        assertTrue(!build.file("modules/jail/architecture-baseline.txt").exists(), "an empty baseline was left behind")
    }

    @Test
    fun GIVEN_api_unit_depending_on_impl_unit_WHEN_architecture_checked_THEN_reports_the_dependency(
        @TempDir rootDirectory: File
    ) {
        val apiScript = """
            $JAVA_MODULE_SCRIPT

            dependencies {
                implementation(project(":modules:impl"))
            }
        """.trimIndent()
        val build = IsolatedProjectsBuild(rootDirectory)
            .online()
            .withRootBuildScript("")
            .withModule(name = "modules:api", buildScript = apiScript)
            .withModule(name = "modules:impl", buildScript = JAVA_MODULE_SCRIPT)
            .withSourceFile("modules/api/README.md", "Publishes what other contexts may call.")

        val result = build.runAndFail(":modules:api:architectureCheck")

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
                api("org.jetbrains.kotlin:kotlin-stdlib:2.1.0")
                api(gradleApi())
            }
        """.trimIndent()
        val build = IsolatedProjectsBuild(rootDirectory)
            .online()
            .withRootBuildScript("")
            .withModule(name = "modules:jail", buildScript = moduleScript)
            .withSourceFile("modules/jail/README.md", "Jails players for a sentence and frees them when it ends.")

        val result = build.runAndFail(":modules:jail:architectureCheck")

        val isDependencyReported = result.output.contains("Declare org.slf4j:slf4j-api with implementation")
        val isRuleNamed = result.output.contains("[ApiConfiguration]")
        assertTrue(
            isDependencyReported && isRuleNamed,
            "the api dependency was not reported:\n${result.output}"
        )
        assertTrue(!result.output.contains("null:unspecified"), result.output)
        assertTrue(!result.output.contains("kotlin-stdlib"), result.output)
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
            .withModule(name = "modules:library", buildScript = moduleScript)
            .withSourceFile("modules/library/README.md", "Publishes the library API.")

        build.run(":modules:library:compileKotlin", ":modules:library:architectureCheck")
        val cachedResult = build.run(":modules:library:compileKotlin", ":modules:library:architectureCheck")

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
            val impl = dependencies.create(project(":modules:impl"))
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
            .withModule(name = "modules:api", buildScript = moduleScript)
            .withModule(name = "modules:impl", buildScript = JAVA_MODULE_SCRIPT)
            .withSourceFile("modules/api/README.md", "Publishes what other contexts may call.")

        build.run(":modules:api:architectureBaseline")
        val cachedResult = build.run(":modules:api:architectureBaseline")

        assertTrue(cachedResult.output.contains("Reusing configuration cache"), cachedResult.output)
        val baseline = build.file("modules/api/architecture-baseline.txt").readLines()
        assertEquals(
            listOf("ApiConfiguration\tbuild.gradle.kts\tapi: org.apache.commons:commons-lang3"),
            baseline
        )
        build.run(":modules:api:architectureCheck")
    }

    @Test
    fun GIVEN_kotlin_unit_WHEN_architecture_lint_planned_THEN_runs_every_linter_task_but_format_and_tests(
        @TempDir rootDirectory: File
    ) {
        val moduleScript = """
            plugins {
                id("org.jetbrains.kotlin.jvm")
                id("ru.astrainteractive.klibs.rules.gradle.linter")
            }
        """.trimIndent()
        val build = IsolatedProjectsBuild(rootDirectory)
            .online()
            .withRootBuildScript("""plugins { id("ru.astrainteractive.klibs.rules.gradle.linter") }""")
            .withModule(name = "modules:library", buildScript = moduleScript)

        val planned = build.run("architectureLint", "--dry-run").output.lines()
            .filter { line -> line.endsWith(" SKIPPED") }
            .map { line -> line.removeSuffix(" SKIPPED") }

        listOf(
            ":architectureGraphCheck",
            ":modules:library:architectureCheck",
            ":modules:library:detekt",
            ":modules:library:detektMain",
            ":modules:library:detektTest",
        ).forEach { task -> assertTrue(task in planned, "$task is not planned:\n$planned") }
        assertTrue(planned.none { task -> task.endsWith(":detektFormat") }, "detektFormat is planned:\n$planned")
        assertTrue(planned.none { task -> task.endsWith(":test") }, "the tests are planned:\n$planned")
    }

    @Test
    fun GIVEN_kotlin_library_with_explicit_api_WHEN_configured_THEN_detekt_runs_the_architecture_rules(
        @TempDir rootDirectory: File
    ) {
        val moduleScript = """
            import dev.detekt.gradle.Detekt

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
            val detektTestArguments = tasks.named<Detekt>("detektTest").get().freeCompilerArgs
            val detektConfig = detektTask.config
            val detektPluginNames = configurations.getByName("detektPlugins").dependencies
                .map { dependency -> "${'$'}{dependency.group}:${'$'}{dependency.name}" }
            val linterReport = layout.buildDirectory.file("linter-report.txt")
            tasks.register("reportLinter") {
                val arguments = detektArguments
                val testArguments = detektTestArguments
                val configFiles = detektConfig
                val pluginNames = detektPluginNames
                val reportFile = linterReport
                outputs.file(reportFile)
                doLast {
                    val lines = listOf(
                        "args=" + arguments.get(),
                        "config=" + configFiles.files.map { file -> file.parentFile.parentFile.name + "/" + file.name },
                        "plugins=" + pluginNames,
                        "testArgs=" + testArguments.get(),
                    )
                    reportFile.get().asFile.writeText(lines.joinToString(separator = "\n"))
                }
            }
        """.trimIndent()
        val build = IsolatedProjectsBuild(rootDirectory)
            .withRootBuildScript("")
            .withModule(name = "modules:library", buildScript = moduleScript)
            .withSourceFile("config/detekt/detekt.yml", PROJECT_IMPORT_LAYOUT)

        build.run(":modules:library:reportLinter")

        val report = build.file("modules/library/build/linter-report.txt").readLines()
        assertEquals("args=[-Xexplicit-api=strict]", report[0])
        assertEquals(
            "config=[build/detekt.yml, build/detekt-architecture.yml, build/architecture-settings.yml, " +
                "config/detekt.yml]",
            report[1]
        )
        assertTrue(report[2].contains("ru.astrainteractive.klibs.rules:detekt"), "rules missing: ${report[2]}")
        assertTrue("-Xexplicit-api" !in report[3], "explicit API leaked into test compilation: ${report[3]}")
    }

    @Test
    fun GIVEN_explicit_api_jvm_library_WHEN_source_set_detekt_runs_THEN_public_visibility_is_required(
        @TempDir rootDirectory: File
    ) {
        val moduleScript = """
            plugins {
                id("org.jetbrains.kotlin.jvm")
                id("ru.astrainteractive.klibs.rules.gradle.linter")
            }

            kotlin { explicitApi() }
        """.trimIndent()
        val build = IsolatedProjectsBuild(rootDirectory)
            .online()
            .withRootBuildScript("")
            .withModule(name = "modules:library", buildScript = moduleScript)
            .withSourceFile(
                "modules/library/src/main/kotlin/com/example/library/api/PublicApi.kt",
                """
                package com.example.library.api

                public interface PublicApi

                """.trimIndent()
            )

        build.run(":modules:library:detektMain", ":modules:library:detektMainSourceSet")
    }

    @Test
    fun GIVEN_explicit_api_kmp_library_WHEN_common_source_set_detekt_runs_THEN_public_visibility_is_required(
        @TempDir rootDirectory: File
    ) {
        val moduleScript = """
            plugins {
                id("org.jetbrains.kotlin.multiplatform")
                id("ru.astrainteractive.klibs.rules.gradle.linter")
            }

            kotlin {
                jvm()
                explicitApi()
            }
        """.trimIndent()
        val build = IsolatedProjectsBuild(rootDirectory)
            .online()
            .withRootBuildScript("")
            .withModule(name = "modules:library", buildScript = moduleScript)
            .withSourceFile(
                "modules/library/src/commonMain/kotlin/com/example/library/api/PublicApi.kt",
                """
                package com.example.library.api

                public interface PublicApi

                """.trimIndent()
            )

        build.run(":modules:library:detektCommonMainSourceSet")
    }

    @Test
    fun GIVEN_latest_main_source_set_with_explicit_api_WHEN_detekt_runs_THEN_public_visibility_is_required(
        @TempDir rootDirectory: File
    ) {
        val moduleScript = """
            plugins {
                id("org.jetbrains.kotlin.multiplatform")
                id("ru.astrainteractive.klibs.rules.gradle.linter")
            }

            kotlin {
                jvm()
                explicitApi()
                sourceSets {
                    val latestMain by creating { dependsOn(commonMain.get()) }
                    jvmMain.get().dependsOn(latestMain)
                    val latestTest by creating { dependsOn(commonTest.get()) }
                    jvmTest.get().dependsOn(latestTest)
                }
            }
        """.trimIndent()
        val build = IsolatedProjectsBuild(rootDirectory)
            .online()
            .withRootBuildScript("")
            .withModule(name = "modules:library", buildScript = moduleScript)
            .withSourceFile(
                "modules/library/src/latestMain/kotlin/com/example/library/api/LatestApi.kt",
                """
                package com.example.library.api

                public interface LatestApi

                """.trimIndent()
            )
            .withSourceFile(
                "modules/library/src/latestTest/kotlin/com/example/library/api/LatestTestApi.kt",
                """
                package com.example.library.api

                public interface LatestTestApi

                """.trimIndent()
            )

        build.run(":modules:library:detektLatestMainSourceSet")
        val testResult = build.runAndFail(":modules:library:detektLatestTestSourceSet")

        assertTrue(testResult.output.contains("RedundantVisibilityModifier"), testResult.output)
    }

    @Test
    fun GIVEN_no_explicit_api_WHEN_main_source_set_detekt_runs_THEN_redundant_public_is_reported(
        @TempDir rootDirectory: File
    ) {
        val moduleScript = """
            plugins {
                id("org.jetbrains.kotlin.jvm")
                id("ru.astrainteractive.klibs.rules.gradle.linter")
            }
        """.trimIndent()
        val build = IsolatedProjectsBuild(rootDirectory)
            .online()
            .withRootBuildScript("")
            .withModule(name = "modules:library", buildScript = moduleScript)
            .withSourceFile(
                "modules/library/src/main/kotlin/com/example/library/api/PublicApi.kt",
                """
                package com.example.library.api

                public interface PublicApi

                """.trimIndent()
            )

        val result = build.runAndFail(":modules:library:detektMain")

        assertTrue(result.output.contains("RedundantVisibilityModifier"), result.output)
    }

    @Test
    fun GIVEN_string_unit_kind_WHEN_configured_THEN_requires_the_enum(@TempDir rootDirectory: File) {
        val moduleScript = """
            $JAVA_MODULE_SCRIPT

            linter {
                unitKind("service")
            }
        """.trimIndent()
        val build = IsolatedProjectsBuild(rootDirectory)
            .online()
            .withRootBuildScript("")
            .withModule(name = "modules:jail", buildScript = moduleScript)

        val result = build.runAndFail(":modules:jail:architectureCheck")

        assertTrue(
            result.output.contains(
                "UnitKind"
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
                testImplementation(project(":modules:impl"))
            }
        """.trimIndent()
        val build = IsolatedProjectsBuild(rootDirectory)
            .online()
            .withRootBuildScript("")
            .withModule(name = "modules:jail", buildScript = moduleScript)
            .withModule(name = "modules:impl", buildScript = JAVA_MODULE_SCRIPT)
            .withSourceFile("modules/jail/README.md", "Jails players for a sentence and frees them when it ends.")

        build.run(":modules:jail:architectureCheck")
    }

    @Test
    fun GIVEN_fake_in_test_fixtures_outside_a_fake_package_WHEN_architecture_checked_THEN_judges_it_as_a_test(
        @TempDir rootDirectory: File
    ) {
        val build = jailBuild(rootDirectory)
            .withSourceFile(
                "modules/jail/src/testFixtures/kotlin/com/example/jail/jail/FakeClock.kt",
                """
                package com.example.jail.jail

                object FakeClock
                """.trimIndent()
            )

        val result = build.runAndFail(":modules:jail:architectureCheck")

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
            .withModule(name = "modules:jail", buildScript = "")
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
    fun GIVEN_kotlin_convention_plugin_WHEN_root_checked_THEN_allows_sources(@TempDir rootDirectory: File) {
        val script = """
            plugins {
                `kotlin-dsl`
                id("ru.astrainteractive.klibs.rules.gradle.linter")
            }
        """.trimIndent()
        val build = IsolatedProjectsBuild(rootDirectory)
            .online()
            .withRootBuildScript(script)
            .withSourceFile("src/main/kotlin/example.java-library.gradle.kts", "plugins { `java-library` }")

        build.run(":architectureCheck")
    }

    @Test
    fun GIVEN_convention_plugin_applied_after_linter_WHEN_root_checked_THEN_allows_sources_and_reuses_cache(
        @TempDir rootDirectory: File
    ) {
        val script = JAVA_MODULE_SCRIPT + """

            apply(plugin = "groovy-gradle-plugin")
        """.trimIndent()
        val build = IsolatedProjectsBuild(rootDirectory)
            .online()
            .withRootBuildScript(script)
            .withSourceFile("src/main/groovy/example.java-library.gradle", "plugins { id 'java-library' }")

        build.run(":architectureCheck")
        val second = build.run(":architectureCheck")

        assertTrue(second.output.contains("Reusing configuration cache"), second.output)
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
            val detektMainArguments = tasks.named<Detekt>("detektMain").get().freeCompilerArgs
            val argumentsReport = layout.buildDirectory.file("detekt-arguments.txt")
            tasks.register("reportDetektArguments") {
                val arguments = detektArguments
                val mainArguments = detektMainArguments
                val reportFile = argumentsReport
                outputs.file(reportFile)
                doLast {
                    reportFile.get().asFile.writeText(
                        listOf(arguments.get(), mainArguments.get()).joinToString(separator = "\n")
                    )
                }
            }
        """.trimIndent()
        val build = IsolatedProjectsBuild(rootDirectory)
            .withRootBuildScript("")
            .withModule(name = "modules:library", buildScript = moduleScript)

        build.run(":modules:library:reportDetektArguments")

        assertEquals(
            "[-Xexplicit-api=warning]\n[-Xexplicit-api=warning]",
            build.file("modules/library/build/detekt-arguments.txt").readText()
        )
    }

    @Test
    fun GIVEN_test_in_another_source_set_WHEN_architecture_checked_THEN_reports_the_expected_set(
        @TempDir rootDirectory: File
    ) {
        val build = IsolatedProjectsBuild(rootDirectory)
            .online()
            .withRootBuildScript("")
            .withModule(name = "modules:jail", buildScript = JAVA_MODULE_SCRIPT)
            .withSourceFile("modules/jail/README.md", "Jails players for a sentence and frees them when it ends.")
            .withSourceFile(
                "modules/jail/src/commonMain/kotlin/com/example/jail/model/Jail.kt",
                "package com.example.jail.model\n\nclass Jail"
            )
            .withSourceFile(
                "modules/jail/src/jvmTest/kotlin/com/example/jail/model/JailTest.kt",
                "package com.example.jail.model\n\nclass JailTest"
            )

        val result = build.runAndFail(":modules:jail:architectureCheck")

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
                exportedApi(project(":modules:core"))
                unrelatedApi("org.apache.commons:commons-lang3:3.14.0")
                unrelatedApi(project(":modules:core"))
            }
        """.trimIndent()
        val build = IsolatedProjectsBuild(rootDirectory)
            .online()
            .withRootBuildScript("")
            .withModule(name = "modules:library", buildScript = moduleScript)
            .withModule(name = "modules:core", buildScript = JAVA_MODULE_SCRIPT)
            .withSourceFile("modules/library/README.md", "Publishes the library API.")
            .withSourceFile("modules/core/README.md", "Provides shared models.")

        val result = build.runAndFail(":modules:library:architectureCheck")

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
            .withModule(name = "modules:library", buildScript = moduleScript)
            .withSourceFile("modules/library/README.md", "Publishes the library API.")

        val result = build.runAndFail(":modules:library:architectureCheck")

        assertTrue(result.output.contains("Declare org.slf4j:slf4j-api"), result.output)
    }

    @Test
    fun GIVEN_another_plugin_with_a_root_rules_version_WHEN_configured_THEN_detekt_keeps_the_linter_rules_version(
        @TempDir rootDirectory: File,
        @TempDir anotherPlugin: File
    ) {
        File(anotherPlugin, "klibs-rules.properties").writeText("version=$ANOTHER_PLUGIN_RULES_VERSION")
        val moduleScript = """
            plugins {
                `java-library`
                id("ru.astrainteractive.klibs.rules.gradle.linter")
            }

            val rulesVersions = configurations.getByName("detektPlugins").dependencies
                .filter { dependency -> dependency.group == "ru.astrainteractive.klibs.rules" }
                .map { dependency -> dependency.version }
            val rulesReport = layout.buildDirectory.file("rules-version.txt")
            tasks.register("reportRulesVersion") {
                val versions = rulesVersions
                val reportFile = rulesReport
                outputs.file(reportFile)
                doLast { reportFile.get().asFile.writeText(versions.joinToString(separator = "\n")) }
            }
        """.trimIndent()
        val build = IsolatedProjectsBuild(rootDirectory)
            .withRootBuildScript("")
            .withModule(name = "library", buildScript = moduleScript)
            .withClasspathAheadOfPlugin(anotherPlugin)

        build.run(":library:reportRulesVersion")

        val versions = build.file("library/build/rules-version.txt").readLines()
        assertEquals(
            1,
            versions.size,
            "expected one rules dependency in detektPlugins: $versions"
        )
        assertTrue(
            ANOTHER_PLUGIN_RULES_VERSION !in versions,
            "the rules version of another plugin was used instead of the linter's own: $versions"
        )
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

        const val ANOTHER_PLUGIN_RULES_VERSION = "0.0.0-another-plugin"
    }
}
