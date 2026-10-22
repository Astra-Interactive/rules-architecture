package ru.astrainteractive.klibs.rules.gradle.linter.model

/**
 * What a unit's own build declares, read at configuration time for the build findings.
 *
 * @property apiDependencies `<configuration>: <dependency>` of every dependency declared in an `api` configuration
 * @property hasFrameworkExport whether a Kotlin Multiplatform framework exports a dependency
 * @property explicitApiMode `strict`, `warning`, or `null` when off or the unit has no Kotlin plugin
 * @property platformDependencies `group:name` of the external production dependencies
 */
internal data class UnitBuild(
    val unitKind: UnitKind,
    val apiDependencies: List<String>,
    val hasFrameworkExport: Boolean,
    val isPublished: Boolean,
    val isKotlin: Boolean,
    val explicitApiMode: String?,
    val platformDependencies: List<String>
)
