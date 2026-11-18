@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.gradle.linter.plugin.gradle

import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
import ru.astrainteractive.klibs.rules.gradle.fixture.IsolatedProjectsBuild
import java.io.File
import kotlin.test.assertFalse
import kotlin.test.assertTrue

internal class VersionCatalogIntegrationTest {
    private fun catalogBuild(directory: File, catalog: String): IsolatedProjectsBuild {
        return IsolatedProjectsBuild(directory)
            .online()
            .withRootBuildScript(ROOT_SCRIPT)
            .withModule(name = "modules:jail", buildScript = MODULE_SCRIPT)
            .withSourceFile("modules/jail/README.md", "Jails players for a sentence and frees them when it ends.")
            .withSourceFile("gradle/libs.versions.toml", catalog)
    }

    @Test
    fun GIVEN_alias_extending_another_in_libraries_WHEN_architecture_checked_THEN_fails_and_reuses_the_cache(
        @TempDir directory: File
    ) {
        val build = catalogBuild(directory, COLLIDING_LIBRARIES)

        val first = build.runAndFail(":architectureCheck")
        val second = build.runAndFail(":architectureCheck")

        assertTrue(first.output.contains("[CatalogAliasPrefix]"), first.output)
        assertTrue(first.output.contains("decompose-experimental-compose"), first.output)
        assertTrue(second.output.contains("Reusing configuration cache"), second.output)
        assertTrue(second.output.contains("[CatalogAliasPrefix]"), second.output)
        assertTrue(second.output.contains("decompose-experimental-compose"), second.output)
    }

    @Test
    fun GIVEN_aliases_that_are_no_segment_prefix_WHEN_architecture_checked_THEN_reports_no_rule(
        @TempDir directory: File
    ) {
        val build = catalogBuild(directory, SEPARATE_ALIASES)

        val result = build.run(":architectureCheck")

        assertFalse(result.output.contains("[CatalogAliasPrefix]"), result.output)
    }

    @Test
    fun GIVEN_colliding_catalog_WHEN_module_checked_THEN_only_the_root_reports_the_rule(@TempDir directory: File) {
        val build = catalogBuild(directory, COLLIDING_LIBRARIES)

        val root = build.runAndFail(":architectureCheck")
        val module = build.run(":modules:jail:architectureCheck")

        assertTrue(root.output.contains("[CatalogAliasPrefix]"), root.output)
        assertFalse(module.output.contains("[CatalogAliasPrefix]"), module.output)
    }

    @Test
    fun GIVEN_bundle_alias_extending_another_WHEN_architecture_checked_THEN_fails_naming_the_bundle(
        @TempDir directory: File
    ) {
        val build = catalogBuild(directory, BUNDLE_CATALOG)

        val result = build.runAndFail(":architectureCheck")

        assertTrue(result.output.contains("[CatalogAliasPrefix]"), result.output)
        assertTrue(result.output.contains("Rename the [bundles] alias net-client"), result.output)
    }

    private companion object {
        val ROOT_SCRIPT = """
            plugins { id("ru.astrainteractive.klibs.rules.gradle.linter") }
        """.trimIndent()

        val MODULE_SCRIPT = """
            plugins {
                `java-library`
                id("ru.astrainteractive.klibs.rules.gradle.linter")
            }
        """.trimIndent()

        val COLLIDING_LIBRARIES = """
            [libraries]
            decompose-compose = { module = "com.example.decompose:compose", version = "3.2.0" }
            decompose-compose-experimental = { module = "com.example.decompose:experimental", version = "3.2.0" }
        """.trimIndent()

        val SEPARATE_ALIASES = """
            [libraries]
            decompose-compose = { module = "com.example.decompose:compose", version = "3.2.0" }
            decompose-composeexperimental = { module = "com.example.decompose:experimental", version = "3.2.0" }

            [plugins]
            kotlin = { id = "org.jetbrains.kotlin.jvm", version = "2.1.0" }
            kotlinjvm = { id = "com.example.kotlin-jvm", version = "1.0.0" }
        """.trimIndent()

        val BUNDLE_CATALOG = """
            [libraries]
            net-core = { module = "com.example.net:core", version = "1.0.0" }

            [bundles]
            net = ["net-core"]
            net_client = ["net-core"]
        """.trimIndent()
    }
}
