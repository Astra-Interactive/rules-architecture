package ru.astrainteractive.klibs.rules.gradle.linter.util

import ru.astrainteractive.klibs.rules.gradle.linter.model.ArchitectureFinding
import ru.astrainteractive.klibs.rules.gradle.linter.model.UnitBuild

private const val BUILD_FILE = "build.gradle.kts"
private const val STRICT = "strict"

private fun UnitBuild.findApiConfigurationFindings(): List<ArchitectureFinding> {
    if (hasFrameworkExport) return emptyList()
    return apiDependencies.map { dependency ->
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

internal fun UnitBuild.findBuildFindings(): List<ArchitectureFinding> {
    return findApiConfigurationFindings() + findExplicitApiFindings()
}
