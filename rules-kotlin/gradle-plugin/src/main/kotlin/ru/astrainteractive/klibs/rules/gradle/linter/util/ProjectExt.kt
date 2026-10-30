@file:Suppress("TooManyFunctions")

package ru.astrainteractive.klibs.rules.gradle.linter.util

import org.gradle.api.Project
import org.gradle.api.artifacts.Configuration
import org.gradle.api.artifacts.Dependency
import org.gradle.api.artifacts.ExternalModuleDependency
import org.gradle.api.artifacts.ProjectDependency
import org.gradle.api.file.FileCollection
import org.gradle.api.provider.Provider
import org.jetbrains.kotlin.gradle.dsl.ExplicitApiMode
import org.jetbrains.kotlin.gradle.dsl.KotlinBaseExtension
import org.jetbrains.kotlin.gradle.dsl.KotlinMultiplatformExtension
import org.jetbrains.kotlin.gradle.plugin.mpp.Framework
import org.jetbrains.kotlin.gradle.plugin.mpp.KotlinNativeTarget
import ru.astrainteractive.klibs.rules.gradle.linter.extension.model.LinterExtension
import ru.astrainteractive.klibs.rules.gradle.linter.model.UnitBuild
import ru.astrainteractive.klibs.rules.gradle.linter.model.UnitKind
import ru.astrainteractive.klibs.rules.gradle.util.hasKotlinPlugin
import ru.astrainteractive.klibs.rules.structure.unit.structure.isPlatformUnitName
import ru.astrainteractive.klibs.rules.structure.unit.structure.isTechnologyUnitName
import java.io.File

private const val ENTRY_POINT_PATH_PREFIX = ":instances:"

private const val CORE_SEGMENT = "core"

private val ENTRY_POINT_PLUGIN_IDS = listOf("application", "com.android.application")

private const val MULTIPLATFORM_PLUGIN_ID = "org.jetbrains.kotlin.multiplatform"

private const val PUBLISH_PLUGIN_ID = "maven-publish"

private const val API_CONFIGURATION = "api"

private const val ANDROID_EXTENSION = "android"

private const val NAMESPACE_GETTER = "getNamespace"

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

/**
 * A sub-context's platform unit (`gui/bukkit`) is wholly that platform; `core/bukkit` holds small platform pieces as
 * packages of `core` and is not.
 */
private fun Project.isPlatformUnit(): Boolean {
    val segments = path.split(':')
    return isPlatformUnitName(segments.last()) && CORE_SEGMENT !in segments
}

/**
 * Call it only when a Kotlin plugin is applied.
 */
internal fun Project.explicitApiMode(): String? {
    return when (extensions.findByType(KotlinBaseExtension::class.java)?.explicitApi) {
        ExplicitApiMode.Strict -> "strict"
        ExplicitApiMode.Warning -> "warning"
        else -> null
    }
}

/**
 * A published library is a unit applying `maven-publish` with strict explicit API mode; explicit API mode alone does
 * not make an application unit a library.
 */
private fun Project.isPublishedLibrary(): Boolean {
    val isStrict = hasKotlinPlugin() && explicitApiMode() == "strict"
    return isStrict && pluginManager.hasPlugin(PUBLISH_PLUGIN_ID)
}

internal fun Project.detectUnitKind(): UnitKind {
    val unitName = path.substringAfterLast(':')
    return when {
        unitName == UnitKind.API.cliName -> UnitKind.API
        unitName == UnitKind.CONTRACT.cliName -> UnitKind.CONTRACT
        isTechnologyUnitName(unitName) -> UnitKind.TECHNOLOGY
        isEntryPointByConvention() -> UnitKind.ENTRY_POINT
        isPlatformUnit() -> UnitKind.TECHNOLOGY
        isPublishedLibrary() -> UnitKind.LIBRARY
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
            configuration.dependencies
                .filterNot { dependency -> dependency.group == null && dependency.name == "unspecified" }
                .map { dependency -> "${configuration.name}: ${dependencyNotation(dependency)}" }
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

/**
 * The unit's technology words: the project's own from [extension] and those its libraries name in its build file.
 */
internal fun Project.findUnitTechnologyWords(extension: LinterExtension): Provider<Set<String>> {
    val libraryWords = provider { findTechnologyWords(externalProductionDependencies()) }
    return extension.technologyWords.zip(libraryWords) { projectWords, words -> projectWords + words }
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

/**
 * The Kotlin sources of the units this one depends on, for the architecture check's up-to-date state.
 */
internal fun Project.dependencyInputs(): List<FileCollection> {
    val root = isolated.rootProject.projectDirectory.asFile
    return productionProjectDependencies().map { path ->
        fileTree(File(root, path.removePrefix(":").replace(':', '/'))) { tree ->
            tree.include("src/**/*.kt")
            tree.exclude("src/*[Tt]est*/**", "**/build/**")
        }
    }
}

/**
 * The `namespace` of an Android unit, read from the `android` extension by reflection so that the plugin does not
 * depend on the Android Gradle plugin; `null` outside Android.
 */
internal fun Project.androidNamespace(): String? {
    val android = extensions.findByName(ANDROID_EXTENSION) ?: return null
    val getter = android.javaClass.methods
        .firstOrNull { method -> method.name == NAMESPACE_GETTER && method.parameterCount == 0 }
        ?: return null
    return getter.invoke(android) as? String
}
