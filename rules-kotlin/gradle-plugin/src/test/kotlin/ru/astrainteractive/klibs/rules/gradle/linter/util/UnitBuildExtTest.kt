@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.gradle.linter.util

import ru.astrainteractive.klibs.rules.gradle.linter.model.ArchitectureFinding
import ru.astrainteractive.klibs.rules.gradle.linter.model.UnitBuild
import ru.astrainteractive.klibs.rules.gradle.linter.model.UnitKind
import kotlin.test.Test
import kotlin.test.assertEquals

class UnitBuildExtTest {
    private val cleanBuild = UnitBuild(
        unitKind = UnitKind.API,
        apiDependencies = emptyList(),
        exportedDependencies = emptySet(),
        isPublished = false,
        isKotlin = true,
        explicitApiMode = null,
        platformDependencies = listOf("androidx.compose.runtime:runtime", "org.jetbrains.kotlinx:kotlinx-coroutines")
    )

    @Test
    fun GIVEN_api_configuration_without_framework_export_WHEN_checked_THEN_reports_each_dependency() {
        val build = cleanBuild.copy(apiDependencies = listOf("api: com.example:core", "jvmMainApi: :core"))

        val findings = build.findBuildFindings()

        assertEquals(listOf("api: com.example:core", "jvmMainApi: :core"), findings.map(ArchitectureFinding::subject))
        assertEquals(
            "Declare com.example:core with implementation: api is only for a type exported through a " +
                "Multiplatform framework.",
            findings.first().message
        )
    }

    @Test
    fun GIVEN_exported_and_unrelated_api_dependencies_WHEN_checked_THEN_reports_only_unrelated() {
        val build = cleanBuild.copy(
            apiDependencies = listOf("api: com.example:core", "api: com.example:other"),
            exportedDependencies = setOf("com.example:core")
        )

        assertEquals(
            listOf("api: com.example:other"),
            build.findBuildFindings().map(ArchitectureFinding::subject)
        )
    }

    @Test
    fun GIVEN_published_kotlin_library_without_strict_explicit_api_WHEN_checked_THEN_reports_it() {
        val warning = cleanBuild.copy(isPublished = true, explicitApiMode = "warning")
        val strict = cleanBuild.copy(isPublished = true, explicitApiMode = "strict")
        val java = cleanBuild.copy(isPublished = true, isKotlin = false)

        val findings = warning.findBuildFindings() + strict.findBuildFindings() + java.findBuildFindings()

        assertEquals(listOf("PublishedLibraryExplicitApi"), findings.map(ArchitectureFinding::rule))
    }

    @Test
    fun GIVEN_api_unit_with_platform_dependencies_WHEN_checked_THEN_reports_each() {
        val build = cleanBuild.copy(
            platformDependencies = listOf(
                "io.papermc.paper:paper-api",
                "androidx.core:core-ktx",
                "androidx.annotation:annotation",
                "com.android.tools:desugar"
            )
        )
        val context = build.copy(unitKind = UnitKind.CONTEXT)

        val findings = build.findBuildFindings() + context.findBuildFindings()

        assertEquals(
            listOf(
                "io.papermc.paper:paper-api",
                "androidx.core:core-ktx",
                "com.android.tools:desugar"
            ),
            findings.map(ArchitectureFinding::subject)
        )
        assertEquals(
            "Move the platform dependency io.papermc.paper:paper-api out of this api unit: the api unit holds " +
                "platform-free code, and its platform unit depends on it.",
            findings.first().message
        )
    }
}
