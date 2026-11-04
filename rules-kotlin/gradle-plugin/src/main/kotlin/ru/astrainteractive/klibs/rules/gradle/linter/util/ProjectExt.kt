@file:Suppress("TooManyFunctions")

package ru.astrainteractive.klibs.rules.gradle.linter.util

import com.android.build.api.variant.ApplicationAndroidComponentsExtension
import org.gradle.api.Project
import org.gradle.api.artifacts.Configuration
import org.gradle.api.artifacts.Dependency
import org.gradle.api.artifacts.ExternalModuleDependency
import org.gradle.api.artifacts.ProjectDependency
import org.gradle.api.artifacts.VersionCatalogsExtension
import org.gradle.api.artifacts.component.ModuleComponentIdentifier
import org.gradle.api.file.FileCollection
import org.gradle.api.plugins.JavaPluginExtension
import org.gradle.api.provider.Provider
import org.jetbrains.kotlin.gradle.ExperimentalKotlinGradlePluginApi
import org.jetbrains.kotlin.gradle.dsl.ExplicitApiMode
import org.jetbrains.kotlin.gradle.dsl.KotlinBaseExtension
import org.jetbrains.kotlin.gradle.dsl.KotlinMultiplatformExtension
import org.jetbrains.kotlin.gradle.dsl.KotlinSingleTargetExtension
import org.jetbrains.kotlin.gradle.plugin.KotlinCompilation
import org.jetbrains.kotlin.gradle.plugin.mpp.Framework
import org.jetbrains.kotlin.gradle.plugin.mpp.KotlinNativeTarget
import ru.astrainteractive.klibs.rules.core.unit.model.ProductKind
import ru.astrainteractive.klibs.rules.core.unit.model.UnitContext
import ru.astrainteractive.klibs.rules.core.unit.model.UnitKind
import ru.astrainteractive.klibs.rules.gradle.linter.extension.model.LinterExtension
import ru.astrainteractive.klibs.rules.gradle.linter.model.AndroidVariantSources
import ru.astrainteractive.klibs.rules.gradle.linter.model.UnitBuild
import ru.astrainteractive.klibs.rules.gradle.util.hasKotlinPlugin
import ru.astrainteractive.klibs.rules.structure.unit.structure.isPlatformUnitName
import ru.astrainteractive.klibs.rules.structure.unit.structure.isTechnologyUnitName
import java.io.File

private const val ENTRY_POINT_PATH_PREFIX = ":instances:"

private const val CORE_SEGMENT = "core"

private const val MIKRO_CORE = "mikro-core"

private val ENTRY_POINT_PLUGIN_IDS = listOf("application", "com.android.application")

private const val MULTIPLATFORM_PLUGIN_ID = "org.jetbrains.kotlin.multiplatform"

private const val PUBLISH_PLUGIN_ID = "maven-publish"

private const val API_CONFIGURATION = "api"

private const val KOTLIN_GROUP = "org.jetbrains.kotlin"

private const val STDLIB_PREFIX = "kotlin-stdlib"

private const val ANDROID_EXTENSION = "android"

private const val ANDROID_APPLICATION_PLUGIN_ID = "com.android.application"

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
 * Publishing identifies the library independently of its role. Strict explicit API is verified separately.
 */
internal fun Project.isPublishedLibrary(): Boolean {
    return pluginManager.hasPlugin(PUBLISH_PLUGIN_ID)
}

private fun detectCoreKind(unitName: String, segments: List<String>, technologyWords: Set<String>): UnitKind {
    return when {
        isPlatformUnitName(unitName) -> UnitKind.CORE_PLATFORM
        "ui" in segments.dropWhile { segment -> segment != CORE_SEGMENT } -> UnitKind.CORE_UI
        isTechnologyUnitName(unitName) || unitName in technologyWords -> UnitKind.TECHNOLOGY
        else -> UnitKind.CORE_API
    }
}

internal fun Project.detectUnitKind(technologyWords: Set<String> = emptySet()): UnitKind {
    if (isEntryPointByConvention()) return UnitKind.ENTRY_POINT
    val unitName = path.substringAfterLast(':')
    val segments = path.split(':')
    if (CORE_SEGMENT in segments) {
        return detectCoreKind(
            unitName,
            segments,
            technologyWords
        )
    }
    return when {
        unitName == UnitKind.API.cliName -> UnitKind.API
        unitName == UnitKind.CONTRACT.cliName -> UnitKind.CONTRACT
        unitName == "dao" -> UnitKind.STORAGE_CONTRACT
        unitName == UnitKind.IMPL.cliName -> UnitKind.IMPL
        unitName == UnitKind.FAKE.cliName -> UnitKind.FAKE
        isTechnologyUnitName(unitName) || unitName in technologyWords -> UnitKind.TECHNOLOGY
        isPlatformUnit() -> UnitKind.PLATFORM
        else -> UnitKind.CONTEXT
    }
}

private fun Project.kotlinCompilations(): List<KotlinCompilation<*>> {
    if (!hasKotlinPlugin()) return emptyList()
    return when (val kotlin = extensions.findByName("kotlin")) {
        is KotlinMultiplatformExtension -> kotlin.targets.flatMap { target -> target.compilations }
        is KotlinSingleTargetExtension<*> -> kotlin.target.compilations.toList()
        else -> emptyList()
    }
}

internal fun Project.registerAndroidVariantSources(): MutableList<AndroidVariantSources> {
    val variants = mutableListOf<AndroidVariantSources>()
    pluginManager.withPlugin(ANDROID_APPLICATION_PLUGIN_ID) {
        extensions.getByType(ApplicationAndroidComponentsExtension::class.java).onVariants { variant ->
            variant.sources.kotlin?.all?.let { directories ->
                variants.add(AndroidVariantSources(variant.name, directories))
            }
        }
    }
    return variants
}

@OptIn(ExperimentalKotlinGradlePluginApi::class)
private fun Project.readCompilation(
    compilation: KotlinCompilation<*>,
    resolveCompose: Boolean
): UnitContext.Compilation {
    val visible = compilation.allAssociatedCompilations + compilation
    val sourceSets = buildMap {
        visible.flatMap { associated -> associated.allKotlinSourceSets }.forEach { sourceSet ->
            val directories = sourceSet.kotlin.srcDirs
                .filterNot { directory -> directory.toPath().startsWith(layout.buildDirectory.get().asFile.toPath()) }
                .map(File::getAbsolutePath)
                .toSet()
            put(sourceSet.name, directories)
        }
    }
    val compiledNames = compilation.allKotlinSourceSets.map { sourceSet -> sourceSet.name }.toSet()
    val compiledSourceSets = sourceSets.filterKeys(compiledNames::contains)
    val configuration = configurations.getByName(compilation.compileDependencyConfigurationName)
    val compose = if (resolveCompose) {
        configuration.incoming.resolutionResult.allComponents.any { component ->
            val id = component.id as? ModuleComponentIdentifier ?: return@any false
            id.group.startsWith("androidx.compose.") || id.group.startsWith("org.jetbrains.compose.")
        }
    } else {
        null
    }
    val dependencies = configuration.allDependencies.withType(ProjectDependency::class.java)
        .map(ProjectDependency::getPath)
        .filter { dependency -> dependency != path }
        .toSet()
    val target = compilation.target
    val platform = if (target is KotlinNativeTarget) {
        "native-${target.konanTarget.name}"
    } else {
        target.platformType.name
    }
    return UnitContext.Compilation(
        "$platform:${target.name}:${compilation.name}",
        sourceSets,
        compilation.name.contains("test", ignoreCase = true),
        compose,
        dependencies,
        compiledSourceSets
    )
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

private fun Project.androidCompilations(variants: List<AndroidVariantSources>): List<UnitContext.Compilation> {
    val buildDirectory = layout.buildDirectory.get().asFile.toPath()
    return variants.map { variant ->
        val directories = variant.kotlinDirectories.get()
            .map { directory -> directory.asFile }
            .filterNot { directory -> directory.toPath().startsWith(buildDirectory) }
            .map(File::getAbsolutePath)
            .toSet()
        UnitContext.Compilation(
            "android:android:${variant.name}",
            mapOf(variant.name to directories),
            false,
            null,
            productionProjectDependencies()
        )
    }
}

private fun isApiConfiguration(name: String): Boolean {
    val isApi = name == API_CONFIGURATION || name.endsWith(API_CONFIGURATION.replaceFirstChar(Char::uppercase))
    return isApi && !name.contains("test", ignoreCase = true)
}

private fun dependencyNotation(dependency: Dependency): String {
    return (dependency as? ProjectDependency)?.path ?: "${dependency.group}:${dependency.name}"
}

/**
 * The Kotlin Gradle plugin puts the standard library into `api` on its own (Android units see it there), and every
 * Kotlin unit exports its types anyway, so it is no dependency a build file chose to export.
 */
private fun isStandardLibrary(dependency: Dependency): Boolean {
    return dependency.group == KOTLIN_GROUP && dependency.name.startsWith(STDLIB_PREFIX)
}

private fun Project.apiDependencies(): List<String> {
    return configurations
        .filter { configuration -> isApiConfiguration(configuration.name) }
        .flatMap { configuration ->
            configuration.dependencies
                .filterNot { dependency -> dependency.group == null && dependency.name == "unspecified" }
                .filterNot(::isStandardLibrary)
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

private fun Project.sourceSetConfigurationOrigins(): Map<String, String> {
    val kotlinSourceSets = when (val kotlin = extensions.findByName("kotlin")) {
        is KotlinMultiplatformExtension -> kotlin.sourceSets.toList()
        is KotlinSingleTargetExtension<*> -> kotlin.sourceSets.toList()
        else -> emptyList()
    }
    val sourceSetConfigurations = buildMap {
        kotlinSourceSets.forEach { sourceSet ->
            listOf(
                sourceSet.apiConfigurationName,
                sourceSet.implementationConfigurationName,
                sourceSet.compileOnlyConfigurationName,
                sourceSet.runtimeOnlyConfigurationName
            ).forEach { configuration -> put(configuration, sourceSet.name) }
        }
    }
    return buildMap {
        extensions.findByType(JavaPluginExtension::class.java)?.sourceSets?.forEach { sourceSet ->
            listOf(
                sourceSet.apiConfigurationName,
                sourceSet.implementationConfigurationName,
                sourceSet.compileOnlyConfigurationName,
                sourceSet.runtimeOnlyConfigurationName
            ).forEach { configuration -> put(configuration, sourceSet.name) }
        }
        putAll(sourceSetConfigurations)
    }
}

/** Direct declaration origins, separate from the compilations that later consume each source set. */
private fun Project.externalDependencyDeclarations(): List<UnitBuild.ExternalDependency> {
    val compilations = kotlinCompilations()
    val origins = sourceSetConfigurationOrigins()
    return configurations.flatMap { configuration ->
        val sourceSet = origins[configuration.name]
        val consumers = compilations.filter { compilation ->
            compilation.allKotlinSourceSets.any { candidate -> candidate.name == sourceSet }
        }
        val production = if (sourceSet != null && consumers.isNotEmpty()) {
            consumers.any { compilation -> !compilation.name.contains("test", ignoreCase = true) }
        } else {
            isProductionConfiguration(configuration.name)
        }
        configuration.dependencies.withType(ExternalModuleDependency::class.java).map { dependency ->
            UnitBuild.ExternalDependency(
                coordinate = "${dependency.group}:${dependency.name}",
                configuration = configuration.name,
                sourceSet = sourceSet,
                production = production,
                test = consumers.any { compilation -> compilation.name.contains("test", ignoreCase = true) },
                targetCompilations = consumers.map { compilation ->
                    val target = compilation.target
                    val platform = if (target is KotlinNativeTarget) {
                        "native-${target.konanTarget.name}"
                    } else {
                        target.platformType.name
                    }
                    UnitBuild.CompilationUse(
                        target = target.name,
                        compilation = compilation.name,
                        platform = platform,
                        test = compilation.name.contains("test", ignoreCase = true)
                    )
                }.toSet()
            )
        }
    }.sortedWith(compareBy(UnitBuild.ExternalDependency::configuration, UnitBuild.ExternalDependency::coordinate))
}

/**
 * Whether the project's version catalog offers mikro-core, whose `propagateCancellationException()` then replaces the
 * inline `CancellationException` check.
 */
internal fun Project.hasMikroCoreInCatalog(): Boolean {
    val catalogs = extensions.findByType(VersionCatalogsExtension::class.java) ?: return false
    return catalogs.any { catalog ->
        catalog.libraryAliases.any { alias ->
            val library = catalog.findLibrary(alias).orElse(null) ?: return@any false
            library.get().module.name == MIKRO_CORE
        }
    }
}

/**
 * The unit's technology words: the project's own from [extension] and those its libraries name in its build file.
 */
internal fun Project.findUnitTechnologyWords(extension: LinterExtension): Provider<Set<String>> {
    val libraryWords = provider { findTechnologyWords(externalProductionDependencies()) }
    return extension.technologyWords.zip(libraryWords) { projectWords, words -> projectWords + words }
}

internal fun Project.readUnitContext(
    extension: LinterExtension,
    rootProductKind: ProductKind?,
    resolveCompose: Boolean = true,
    androidVariants: List<AndroidVariantSources> = emptyList()
): UnitContext {
    val localProduct = extension.configuredProductKind
    require(localProduct == null || rootProductKind == null || localProduct == rootProductKind) {
        "Conflicting linter.productKind for $path: root declares $rootProductKind, unit declares $localProduct"
    }
    val technologies = findUnitTechnologyWords(extension).get()
    val compilations = kotlinCompilations().map { compilation -> readCompilation(compilation, resolveCompose) }
        .ifEmpty { androidCompilations(androidVariants) }
    val context = UnitContext(
        extension.configuredUnitKind ?: detectUnitKind(technologies),
        localProduct ?: rootProductKind,
        isPublishedLibrary(),
        technologies,
        compilations
    )
    return extension.configuredUnitPolicy?.let { policy -> context.copy(policy = policy) } ?: context
}

internal fun Project.readUnitBuild(unitKind: UnitKind, productKind: ProductKind? = null): UnitBuild {
    val isKotlin = hasKotlinPlugin()
    return UnitBuild(
        unitKind = unitKind,
        apiDependencies = apiDependencies(),
        exportedDependencies = exportedDependencies(),
        isPublished = pluginManager.hasPlugin(PUBLISH_PLUGIN_ID),
        isKotlin = isKotlin,
        explicitApiMode = if (isKotlin) explicitApiMode() else null,
        platformDependencies = externalProductionDependencies(),
        productKind = productKind,
        externalDependencies = externalDependencyDeclarations()
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
