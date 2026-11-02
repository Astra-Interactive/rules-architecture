package ru.astrainteractive.klibs.rules.gradle.linter.model

import ru.astrainteractive.klibs.rules.core.unit.model.ProductKind
import ru.astrainteractive.klibs.rules.core.unit.model.UnitKind

/**
 * What a unit's own build declares, read at configuration time for the build findings.
 *
 * @property apiDependencies `<configuration>: <dependency>` of the directly declared `api` dependencies
 * @property exportedDependencies `group:name` or project paths exported through a Multiplatform framework
 * @property explicitApiMode `strict`, `warning`, or `null` when off or the unit has no Kotlin plugin
 * @property platformDependencies Legacy `group:name` values for callers without origin data
 * @property externalDependencies Direct declarations with their Gradle configuration and Kotlin source-set origin
 */
internal data class UnitBuild(
    val unitKind: UnitKind,
    val apiDependencies: List<String>,
    val exportedDependencies: Set<String>,
    val isPublished: Boolean,
    val isKotlin: Boolean,
    val explicitApiMode: String?,
    val platformDependencies: List<String>,
    val productKind: ProductKind?,
    val externalDependencies: List<ExternalDependency>
) {
    internal data class ExternalDependency(
        val coordinate: String,
        val configuration: String,
        val sourceSet: String?,
        val production: Boolean,
        val test: Boolean,
        val targetCompilations: Set<CompilationUse>
    ) {
        val compilationPlatforms: Set<String> = targetCompilations
            .filterNot(CompilationUse::test)
            .map(CompilationUse::platform)
            .toSet()

        constructor(
            coordinate: String,
            configuration: String,
            sourceSet: String?,
            production: Boolean,
            compilationPlatforms: Set<String>
        ) : this(
            coordinate,
            configuration,
            sourceSet,
            production,
            !production,
            compilationPlatforms.map { platform ->
                CompilationUse(
                    platform,
                    "fixture",
                    platform,
                    false
                )
            }.toSet()
        )
    }

    internal data class CompilationUse(
        val target: String,
        val compilation: String,
        val platform: String,
        val test: Boolean
    )

    constructor(
        unitKind: UnitKind,
        apiDependencies: List<String>,
        exportedDependencies: Set<String>,
        isPublished: Boolean,
        isKotlin: Boolean,
        explicitApiMode: String?,
        platformDependencies: List<String>
    ) : this(
        unitKind,
        apiDependencies,
        exportedDependencies,
        isPublished,
        isKotlin,
        explicitApiMode,
        platformDependencies,
        null,
        emptyList()
    )
}
