package ru.astrainteractive.klibs.rules.gradle.linter.util

import ru.astrainteractive.klibs.rules.core.unit.model.ProductKind
import ru.astrainteractive.klibs.rules.core.unit.model.UnitKind
import ru.astrainteractive.klibs.rules.gradle.linter.model.ArchitectureFinding
import ru.astrainteractive.klibs.rules.gradle.linter.model.UnitBuild

private const val BUILD_FILE = "build.gradle.kts"
private const val STRICT = "strict"

private val PLATFORM_GROUP_PREFIXES = listOf(
    "io.papermc",
    "org.spigotmc",
    "org.bukkit",
    "net.md-5",
    "com.velocitypowered",
    "net.fabricmc",
    "net.minecraftforge",
    "net.neoforged",
    "com.android",
    "androidx.",
)

private val PLATFORM_FREE_ANDROIDX_PREFIXES = listOf(
    "androidx.compose",
    "androidx.annotation",
    "androidx.collection",
)

private fun UnitBuild.findApiConfigurationFindings(): List<ArchitectureFinding> {
    return apiDependencies
        .filterNot { dependency -> dependency.substringAfter(": ") in exportedDependencies }
        .map { dependency ->
            ArchitectureFinding(
                rule = "ApiConfiguration",
                path = BUILD_FILE,
                subject = dependency,
                message = "Declare ${dependency.substringAfter(": ")} with implementation: api is only for a type " +
                    "exported through a Multiplatform framework."
            )
        }
}

private fun UnitBuild.findExplicitApiFindings(): List<ArchitectureFinding> {
    if (!isKotlin) return emptyList()
    if (!isPublished && explicitApiMode != null) {
        val finding = ArchitectureFinding(
            rule = "UnpublishedExplicitApi",
            path = BUILD_FILE,
            subject = "explicitApi",
            message = "Switch off explicit API mode in this unit: only a published library uses it, application " +
                "and plugin units never do."
        )
        return listOf(finding)
    }
    if (!isPublished || explicitApiMode == STRICT) return emptyList()
    val finding = ArchitectureFinding(
        rule = "PublishedLibraryExplicitApi",
        path = BUILD_FILE,
        subject = "explicitApi",
        message = "Switch on kotlin { explicitApi() } in this published library: every public declaration of a " +
            "library is deliberate."
    )
    return listOf(finding)
}

private fun isPlatformGroup(dependency: String): Boolean {
    val group = dependency.substringBefore(':')
    val isPlatform = PLATFORM_GROUP_PREFIXES.any(group::startsWith)
    val isPlatformFree = PLATFORM_FREE_ANDROIDX_PREFIXES.any(group::startsWith)
    return isPlatform && !isPlatformFree
}

private fun UnitBuild.ExternalDependency.isAndroidOrigin(): Boolean {
    if (sourceSet == null || sourceSet == "commonMain") return false
    return compilationPlatforms.isNotEmpty() && compilationPlatforms.all { platform ->
        platform == "androidJvm"
    }
}

private fun UnitBuild.ExternalDependency.isAndroidDependency(): Boolean {
    val group = coordinate.substringBefore(':')
    return group.startsWith("androidx.") || group.startsWith("com.android")
}

private fun UnitBuild.ExternalDependency.isOriginUnknown(): Boolean {
    if (sourceSet == null) return true
    return sourceSet != "commonMain" && compilationPlatforms.isEmpty()
}

private fun UnitBuild.findTypedPlatformDependencyFinding(
    dependency: UnitBuild.ExternalDependency
): ArchitectureFinding? {
    val checkedRoles = setOf(
        UnitKind.API,
        UnitKind.CORE_API,
        UnitKind.CORE
    )
    if (unitKind !in checkedRoles) return null
    val subject = "${dependency.configuration}: ${dependency.coordinate}"
    if (productKind == ProductKind.APP && dependency.isAndroidDependency() && dependency.isAndroidOrigin()) return null
    val missingProduct = productKind == null
    val missingOrigin = productKind == ProductKind.APP && dependency.isOriginUnknown()
    if (missingProduct || missingOrigin) {
        return ArchitectureFinding(
            rule = "PlatformDependencyMetadata",
            path = BUILD_FILE,
            subject = subject,
            message = "Cannot verify the platform origin of ${dependency.coordinate}: configuration " +
                "${dependency.configuration} needs a known product kind, declaring source set and production " +
                "compilation target. Supply the missing build metadata."
        )
    }
    val message = if (productKind == ProductKind.PLATFORM_PLUGIN) {
        "Move the platform dependency ${dependency.coordinate} out of this api unit: the api unit holds " +
            "platform-free code, and its platform unit depends on it."
    } else {
        "Move ${dependency.coordinate} to its matching platform source set: declaration in " +
            "${dependency.sourceSet} is not permitted for $unitKind. Keep common contracts and models platform-free."
    }
    return ArchitectureFinding(
        rule = "ApiUnitPlatformDependency",
        path = BUILD_FILE,
        subject = subject,
        message = message
    )
}

private fun UnitBuild.findPlatformDependencyFindings(): List<ArchitectureFinding> {
    if (isPublished) return emptyList()
    if (externalDependencies.isNotEmpty()) {
        return externalDependencies
            .filter(UnitBuild.ExternalDependency::production)
            .filter { dependency -> isPlatformGroup(dependency.coordinate) }
            .mapNotNull(::findTypedPlatformDependencyFinding)
    }
    if (unitKind != UnitKind.API && unitKind != UnitKind.CORE_API) return emptyList()
    return platformDependencies.filter(::isPlatformGroup).map { dependency ->
        ArchitectureFinding(
            rule = "ApiUnitPlatformDependency",
            path = BUILD_FILE,
            subject = dependency,
            message = "Move the platform dependency $dependency out of this api unit: the api unit holds " +
                "platform-free code, and its platform unit depends on it."
        )
    }
}

internal fun UnitBuild.findBuildFindings(): List<ArchitectureFinding> {
    return findApiConfigurationFindings() + findExplicitApiFindings() + findPlatformDependencyFindings()
}
