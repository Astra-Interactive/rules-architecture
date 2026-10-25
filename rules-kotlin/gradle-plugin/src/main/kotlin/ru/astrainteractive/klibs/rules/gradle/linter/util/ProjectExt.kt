package ru.astrainteractive.klibs.rules.gradle.linter.util

import org.gradle.api.Project
import org.gradle.api.artifacts.Configuration
import org.gradle.api.artifacts.Dependency
import org.gradle.api.artifacts.ExternalModuleDependency
import org.gradle.api.artifacts.ProjectDependency
import org.jetbrains.kotlin.gradle.dsl.KotlinMultiplatformExtension
import org.jetbrains.kotlin.gradle.plugin.mpp.Framework
import org.jetbrains.kotlin.gradle.plugin.mpp.KotlinNativeTarget
import ru.astrainteractive.klibs.rules.gradle.linter.model.UnitBuild
import ru.astrainteractive.klibs.rules.gradle.linter.model.UnitKind
import ru.astrainteractive.klibs.rules.gradle.util.hasKotlinPlugin

private const val ENTRY_POINT_PATH_PREFIX = ":instances:"

private val ENTRY_POINT_PLUGIN_IDS = listOf("application", "com.android.application")

private val TECHNOLOGY_UNIT_NAMES = setOf(
    "cache4k",
    "datastore",
    "exposed",
    "file",
    "krate",
    "ktor",
    "memory",
    "okhttp",
    "retrofit",
    "room",
    "sqldelight",
)

private const val MULTIPLATFORM_PLUGIN_ID = "org.jetbrains.kotlin.multiplatform"

private const val PUBLISH_PLUGIN_ID = "maven-publish"

private const val API_CONFIGURATION = "api"

private val PRODUCTION_CONFIGURATION_SUFFIXES = listOf(
    "implementation",
    "api",
    "compileOnly",
    "runtimeOnly"
)

private fun Project.isEntryPointByConvention(): Boolean {
    return ENTRY_POINT_PLUGIN_IDS
        .any(pluginManager::hasPlugin)
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
        .map(ProjectDependency::getPath)
        .filter { dependencyPath -> dependencyPath != path }
        .toSet()
}

private fun isApiConfiguration(name: String): Boolean {
    val isApi = name == API_CONFIGURATION || name.endsWith(API_CONFIGURATION.replaceFirstChar(Char::uppercase))
    return isApi && !name.contains("test", ignoreCase = true)
}

private fun dependencyNotation(dependency: Dependency): String {
    return (dependency as? ProjectDependency)?.path ?: "${dependency.group}:${dependency.name}"
}

private fun Project.apiDependencies(): List<String> {
    return configurations
        .filter { configuration -> isApiConfiguration(configuration.name) }
        .flatMap { configuration ->
            configuration.dependencies.map { dependency ->
                "${configuration.name}: ${dependencyNotation(dependency)}"
            }
        }
        .sorted()
}

private fun Configuration.dependencyClosure(): Set<Dependency> {
    return dependencies.plus(extendsFrom.flatMap(Configuration::dependencyClosure)).toSet()
}

private fun Project.exportedDependencies(): Set<String> {
    if (!pluginManager.hasPlugin(MULTIPLATFORM_PLUGIN_ID)) return emptySet()
    val kotlin = extensions.findByType(KotlinMultiplatformExtension::class.java) ?: return emptySet()
    val exportNames = kotlin.targets.withType(KotlinNativeTarget::class.java)
        .flatMap { target -> target.binaries.withType(Framework::class.java) }
        .map(Framework::exportConfigurationName)
        .toSet()
    return configurations
        .filter { configuration -> configuration.name in exportNames }
        .flatMap(Configuration::dependencyClosure)
        .map(::dependencyNotation)
        .toSet()
}

private fun Project.externalProductionDependencies(): List<String> {
    return configurations
        .filter { configuration -> isProductionConfiguration(configuration.name) }
        .flatMap { configuration -> configuration.dependencies.withType(ExternalModuleDependency::class.java) }
        .map { dependency -> "${dependency.group}:${dependency.name}" }
        .distinct()
        .sorted()
}

internal fun Project.readUnitBuild(unitKind: UnitKind): UnitBuild {
    val isKotlin = hasKotlinPlugin()
    return UnitBuild(
        unitKind = unitKind,
        apiDependencies = apiDependencies(),
        exportedDependencies = exportedDependencies(),
        isPublished = pluginManager.hasPlugin(PUBLISH_PLUGIN_ID),
        isKotlin = isKotlin,
        explicitApiMode = if (isKotlin) explicitApiMode() else null,
        platformDependencies = externalProductionDependencies()
    )
}
