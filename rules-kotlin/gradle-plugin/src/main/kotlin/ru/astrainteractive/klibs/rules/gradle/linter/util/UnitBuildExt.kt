package ru.astrainteractive.klibs.rules.gradle.linter.util

import ru.astrainteractive.klibs.rules.gradle.linter.model.ArchitectureFinding
import ru.astrainteractive.klibs.rules.gradle.linter.model.UnitBuild
import ru.astrainteractive.klibs.rules.gradle.linter.model.UnitKind

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
    if (!isPublished || !isKotlin || explicitApiMode == STRICT) return emptyList()
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

private fun UnitBuild.findPlatformDependencyFindings(): List<ArchitectureFinding> {
    if (unitKind != UnitKind.API) return emptyList()
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
