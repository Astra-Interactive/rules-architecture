package ru.astrainteractive.klibs.rules.gradle.linter.model

/**
 * What a unit's own build declares, read at configuration time for the build findings.
 *
 * @property apiDependencies `<configuration>: <dependency>` of every dependency declared in an `api` configuration
 * @property exportedDependencies `group:name` or project paths exported through a Multiplatform framework
 * @property explicitApiMode `strict`, `warning`, or `null` when off or the unit has no Kotlin plugin
 * @property platformDependencies `group:name` of the external production dependencies
 */
internal data class UnitBuild(
    val unitKind: UnitKind,
    val apiDependencies: List<String>,
    val exportedDependencies: Set<String>,
    val isPublished: Boolean,
    val isKotlin: Boolean,
    val explicitApiMode: String?,
    val platformDependencies: List<String>
)
