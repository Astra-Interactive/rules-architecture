@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.gradle.linter.util

import ru.astrainteractive.klibs.rules.core.unit.model.ProductKind
import ru.astrainteractive.klibs.rules.core.unit.model.UnitKind
import ru.astrainteractive.klibs.rules.gradle.linter.model.ArchitectureFinding
import ru.astrainteractive.klibs.rules.gradle.linter.model.UnitBuild
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
    fun GIVEN_core_android_dependency_declared_in_android_source_set_WHEN_checked_THEN_allowed() {
        val build = cleanBuild.copy(
            unitKind = UnitKind.CORE,
            productKind = ProductKind.APP,
            externalDependencies = listOf(
                UnitBuild.ExternalDependency(
                    coordinate = "androidx.core:core-splashscreen",
                    configuration = "androidMainImplementation",
                    sourceSet = "androidMain",
                    production = true,
                    compilationPlatforms = setOf("androidJvm")
                )
            )
        )

        assertEquals(emptyList(), build.findBuildFindings())
    }

    @Test
    fun GIVEN_core_common_and_unknown_android_dependencies_WHEN_checked_THEN_reports_both_origins() {
        val build = cleanBuild.copy(
            unitKind = UnitKind.CORE,
            productKind = ProductKind.APP,
            externalDependencies = listOf(
                UnitBuild.ExternalDependency(
                    "androidx.core:core-splashscreen",
                    "commonMainImplementation",
                    "commonMain",
                    true,
                    setOf("androidJvm")
                ),
                UnitBuild.ExternalDependency(
                    "androidx.core:core-ktx",
                    "customImplementation",
                    null,
                    true,
                    emptySet()
                ),
                UnitBuild.ExternalDependency(
                    "androidx.core:core-ktx",
                    "androidTestImplementation",
                    "androidTest",
                    false,
                    setOf("androidJvm")
                )
            )
        )

        assertEquals(
            listOf(
                "commonMainImplementation: androidx.core:core-splashscreen",
                "customImplementation: androidx.core:core-ktx"
            ),
            build.findBuildFindings().map(ArchitectureFinding::subject)
        )
        assertEquals(
            listOf("ApiUnitPlatformDependency", "PlatformDependencyMetadata"),
            build.findBuildFindings().map(ArchitectureFinding::rule)
        )
    }

    @Test
    fun GIVEN_app_api_android_dependency_WHEN_checked_THEN_platform_origin_is_allowed() {
        val build = cleanBuild.copy(
            productKind = ProductKind.APP,
            externalDependencies = listOf(
                UnitBuild.ExternalDependency(
                    "androidx.core:core-splashscreen",
                    "androidMainImplementation",
                    "androidMain",
                    true,
                    setOf("androidJvm")
                )
            )
        )

        assertEquals(emptyList(), build.findBuildFindings())
        assertEquals(emptyList(), build.copy(unitKind = UnitKind.CORE_API).findBuildFindings())
    }

    @Test
    fun GIVEN_android_entry_point_platform_dependencies_WHEN_checked_THEN_no_api_boundary_is_applied() {
        val build = cleanBuild.copy(
            unitKind = UnitKind.ENTRY_POINT,
            productKind = ProductKind.APP,
            externalDependencies = listOf(
                UnitBuild.ExternalDependency(
                    "androidx.core:core-splashscreen",
                    "implementation",
                    "main",
                    true,
                    setOf("androidJvm")
                ),
                UnitBuild.ExternalDependency(
                    "androidx.core:core-ktx",
                    "debugImplementation",
                    null,
                    true,
                    emptySet()
                )
            )
        )

        assertEquals(emptyList(), build.findBuildFindings())
    }

    @Test
    fun GIVEN_common_app_api_and_platform_plugin_api_WHEN_checked_THEN_keeps_contract_boundary() {
        val coordinate = "androidx.core:core-splashscreen"
        val android = UnitBuild.ExternalDependency(
            coordinate,
            "androidMainImplementation",
            "androidMain",
            true,
            setOf("androidJvm")
        )
        val common = UnitBuild.ExternalDependency(
            coordinate,
            "commonMainImplementation",
            "commonMain",
            true,
            setOf("androidJvm")
        )
        val app = cleanBuild.copy(productKind = ProductKind.APP, externalDependencies = listOf(common))
        val plugin = cleanBuild.copy(
            productKind = ProductKind.PLATFORM_PLUGIN,
            externalDependencies = listOf(android)
        )

        assertEquals(
            listOf("commonMainImplementation: $coordinate", "androidMainImplementation: $coordinate"),
            (app.findBuildFindings() + plugin.findBuildFindings()).map(ArchitectureFinding::subject)
        )
    }

    @Test
    fun GIVEN_custom_source_set_shared_with_ios_WHEN_checked_THEN_only_android_branch_is_allowed() {
        val coordinate = "androidx.core:core-ktx"
        val build = cleanBuild.copy(
            unitKind = UnitKind.CORE,
            productKind = ProductKind.APP,
            externalDependencies = listOf(
                UnitBuild.ExternalDependency(
                    coordinate,
                    "mobileMainImplementation",
                    "mobileMain",
                    true,
                    setOf("androidJvm", "native-ios_arm64")
                ),
                UnitBuild.ExternalDependency(
                    coordinate,
                    "androidSharedMainImplementation",
                    "androidSharedMain",
                    true,
                    setOf("androidJvm")
                )
            )
        )

        assertEquals(
            listOf("mobileMainImplementation: $coordinate"),
            build.findBuildFindings().map(ArchitectureFinding::subject)
        )
    }

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
    fun GIVEN_unpublished_kotlin_unit_with_explicit_api_WHEN_checked_THEN_reports_it() {
        val strict = cleanBuild.copy(explicitApiMode = "strict")
        val warning = cleanBuild.copy(explicitApiMode = "warning")
        val java = cleanBuild.copy(isKotlin = false, explicitApiMode = "strict")

        val findings = strict.findBuildFindings() + warning.findBuildFindings() + java.findBuildFindings()

        assertEquals(
            listOf("UnpublishedExplicitApi", "UnpublishedExplicitApi"),
            findings.map(ArchitectureFinding::rule)
        )
        assertEquals(
            "Switch off explicit API mode in this unit: only a published library uses it, application and plugin " +
                "units never do.",
            findings.first().message
        )
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
