package ru.astrainteractive.klibs.rules.gradle.linter.util

import org.gradle.api.Project
import org.gradle.api.artifacts.ProjectDependency
import ru.astrainteractive.klibs.rules.gradle.linter.model.UnitKind
import ru.astrainteractive.klibs.rules.gradle.util.hasKotlinPlugin

private const val ENTRY_POINT_PATH_PREFIX = ":instances:"

private val ENTRY_POINT_PLUGIN_IDS = listOf("application", "com.android.application")

private val TECHNOLOGY_UNIT_NAMES = setOf(
    "cache4k",
    "datastore",
    "exposed",
    "krate",
    "ktor",
    "okhttp",
    "retrofit",
    "room",
    "sqldelight",
)

private val PRODUCTION_CONFIGURATION_SUFFIXES = listOf(
    "implementation",
    "api",
    "compileOnly",
    "runtimeOnly"
)

private fun Project.isEntryPointByConvention(): Boolean {
    return ENTRY_POINT_PLUGIN_IDS
        .any { pluginId -> pluginManager.hasPlugin(pluginId) }
        .or(path.startsWith(ENTRY_POINT_PATH_PREFIX))
}

internal fun Project.detectUnitKind(): UnitKind {
    val unitName = path.substringAfterLast(':')
    return when {
        unitName == UnitKind.API.cliName -> UnitKind.API
        unitName == UnitKind.CONTRACT.cliName -> UnitKind.CONTRACT
        unitName in TECHNOLOGY_UNIT_NAMES -> UnitKind.TECHNOLOGY
        isEntryPointByConvention() -> UnitKind.ENTRY_POINT
        hasKotlinPlugin() && explicitApiMode() == "strict" -> UnitKind.LIBRARY
        else -> UnitKind.CONTEXT
    }
}

private fun isProductionConfiguration(name: String): Boolean {
    return PRODUCTION_CONFIGURATION_SUFFIXES
        .any { suffix -> name.endsWith(suffix, ignoreCase = true) }
        .and(!name.contains("test", ignoreCase = true))
}

internal fun Project.productionProjectDependencies(): Set<String> {
    return configurations
        .filter { configuration -> isProductionConfiguration(configuration.name) }
        .flatMap { configuration -> configuration.dependencies.withType(ProjectDependency::class.java) }
        .map { dependency -> dependency.path }
        .filter { dependencyPath -> dependencyPath != path }
        .toSet()
}
