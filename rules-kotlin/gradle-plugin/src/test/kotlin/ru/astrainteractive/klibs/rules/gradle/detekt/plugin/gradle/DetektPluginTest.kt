@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.gradle.detekt.plugin.gradle

import org.gradle.api.JavaVersion
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
import ru.astrainteractive.klibs.rules.gradle.fixture.IsolatedProjectsBuild
import java.io.File

class DetektPluginTest {
    private fun buildWithTwoModules(rootDirectory: File): IsolatedProjectsBuild {
        val moduleScript = """
            import dev.detekt.gradle.Detekt

            plugins {
                `java-library`
                id("ru.astrainteractive.klibs.rules.gradle.detekt")
            }

            val detektConfig = tasks.named<Detekt>("detekt").get().config
            val configReport = layout.buildDirectory.file("$CONFIG_REPORT_NAME")
            tasks.register("reportDetektConfig") {
                val configFiles = detektConfig
                val reportFile = configReport
                inputs.files(configFiles)
                outputs.file(reportFile)
                doLast {
                    val paths = configFiles.files.joinToString(separator = "\n") { file -> file.absolutePath }
                    reportFile.get().asFile.writeText(paths)
                }
            }
        """.trimIndent()
        return IsolatedProjectsBuild(rootDirectory)
            .withRootBuildScript("")
            .withModule(name = "moduleA", buildScript = moduleScript)
            .withModule(name = "moduleB", buildScript = moduleScript)
    }

    private fun IsolatedProjectsBuild.reportedConfigFile(moduleName: String): File {
        val reported = file("$moduleName/build/$CONFIG_REPORT_NAME").readLines()
        assertEquals(
            1,
            reported.size,
            "expected exactly one detekt configuration file in $moduleName"
        )
        return File(reported.first()).canonicalFile
    }

    @Test
    fun GIVEN_two_modules_WHEN_detekt_configured_THEN_both_point_at_one_config_in_the_root_build_directory(
        @TempDir rootDirectory: File
    ) {
        val build = buildWithTwoModules(rootDirectory)
        build.run(":moduleA:reportDetektConfig", ":moduleB:reportDetektConfig")

        val configFile = build.reportedConfigFile("moduleA")
        assertEquals(configFile, build.reportedConfigFile("moduleB"))
        assertEquals(build.file("build/detekt/detekt.yml").canonicalFile, configFile)
        assertTrue(
            configFile.readText().contains("ktlint:"),
            "the bundled detekt configuration was not written into $configFile"
        )
    }

    @Test
    fun GIVEN_two_modules_WHEN_detekt_configured_THEN_one_config_file_exists_in_the_whole_build(
        @TempDir rootDirectory: File
    ) {
        val build = buildWithTwoModules(rootDirectory)
        build.run(":moduleA:reportDetektConfig", ":moduleB:reportDetektConfig")

        val writtenConfigurations = rootDirectory.walkTopDown()
            .filter { file -> file.name == "detekt.yml" }
            .map(File::getCanonicalFile)
            .toList()
        assertEquals(
            listOf(build.file("build/detekt/detekt.yml").canonicalFile),
            writtenConfigurations
        )
    }

    @Test
    fun GIVEN_deleted_build_directory_WHEN_configuration_cache_is_hit_THEN_config_file_is_written_again(
        @TempDir rootDirectory: File
    ) {
        val build = buildWithTwoModules(rootDirectory)
        build.run(":moduleA:reportDetektConfig")
        val configFile = build.file("build/detekt/detekt.yml")
        assertTrue(configFile.delete(), "could not delete $configFile")

        val result = build.run(":moduleA:reportDetektConfig")

        assertTrue(
            result.output.contains("Reusing configuration cache"),
            "the second run did not hit the configuration cache:\n${result.output}"
        )
        assertTrue(configFile.isFile, "$configFile was not written again on a configuration cache hit")
    }

    private fun jvmTargetBuild(rootDirectory: File, moduleScript: String): IsolatedProjectsBuild {
        return IsolatedProjectsBuild(rootDirectory)
            .withRootBuildScript("")
            .withModule(name = "library", buildScript = moduleScript)
    }

    private fun jvmTargetScript(applyKotlin: Boolean, configuration: String): String {
        return """
            import dev.detekt.gradle.Detekt

            plugins {
                ${if (applyKotlin) """id("org.jetbrains.kotlin.jvm")""" else "`java-library`"}
                id("ru.astrainteractive.klibs.rules.gradle.detekt")
            }

            $configuration

            val detektJvmTarget = tasks.named<Detekt>("detekt").get().jvmTarget
            val jvmTargetReport = layout.buildDirectory.file("$JVM_TARGET_REPORT_NAME")
            tasks.register("reportJvmTarget") {
                val jvmTarget = detektJvmTarget
                val reportFile = jvmTargetReport
                outputs.file(reportFile)
                doLast { reportFile.get().asFile.writeText(jvmTarget.get()) }
            }
        """.trimIndent()
    }

    private fun IsolatedProjectsBuild.reportedJvmTarget(): String {
        run(":library:reportJvmTarget")
        return file("library/build/$JVM_TARGET_REPORT_NAME").readText()
    }

    @Test
    fun GIVEN_kotlin_jvm_target_WHEN_detekt_configured_THEN_resolves_types_against_it(
        @TempDir rootDirectory: File
    ) {
        val script = jvmTargetScript(
            applyKotlin = true,
            configuration = """
                kotlin {
                    compilerOptions {
                        jvmTarget.set(org.jetbrains.kotlin.gradle.dsl.JvmTarget.JVM_11)
                    }
                }
            """.trimIndent()
        )

        assertEquals("11", jvmTargetBuild(rootDirectory, script).reportedJvmTarget())
    }

    @Test
    fun GIVEN_project_without_kotlin_WHEN_detekt_configured_THEN_resolves_types_against_the_running_jvm(
        @TempDir rootDirectory: File
    ) {
        val script = jvmTargetScript(applyKotlin = false, configuration = "")

        assertEquals(
            JavaVersion.current().majorVersion,
            jvmTargetBuild(rootDirectory, script).reportedJvmTarget()
        )
    }

    @Test
    fun GIVEN_jvm_target_in_extension_WHEN_detekt_configured_THEN_it_wins_over_the_kotlin_target(
        @TempDir rootDirectory: File
    ) {
        val script = jvmTargetScript(
            applyKotlin = true,
            configuration = """
                kotlin {
                    compilerOptions {
                        jvmTarget.set(org.jetbrains.kotlin.gradle.dsl.JvmTarget.JVM_11)
                    }
                }

                detektRules {
                    jvmTarget.set("21")
                }
            """.trimIndent()
        )

        assertEquals("21", jvmTargetBuild(rootDirectory, script).reportedJvmTarget())
    }

    private companion object {
        const val CONFIG_REPORT_NAME = "detekt-config-path.txt"
        const val JVM_TARGET_REPORT_NAME = "detekt-jvm-target.txt"
    }
}
