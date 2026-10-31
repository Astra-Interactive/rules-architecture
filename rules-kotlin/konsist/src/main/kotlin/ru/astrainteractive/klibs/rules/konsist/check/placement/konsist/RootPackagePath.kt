package ru.astrainteractive.klibs.rules.konsist.check.placement.konsist

import ru.astrainteractive.klibs.rules.konsist.check.konsist.ArchitectureCheck
import ru.astrainteractive.klibs.rules.konsist.check.konsist.UnitSources
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import ru.astrainteractive.klibs.rules.konsist.layout.model.UnitKind

/**
 * Reports a unit whose root package does not end with the unit's name taken from its path: `modules/jail` ends
 * at `jail`, `modules/event-bukkit` at `event`, `components/shop/payment/impl` at `payment.impl`. The entry point,
 * named for the application, and a published library are not judged.
 */
internal class RootPackagePath : ArchitectureCheck {
    override val unitKinds: Set<UnitKind> = UnitKind.entries.toSet() - UnitKind.ENTRY_POINT - UnitKind.LIBRARY

    override fun check(sources: UnitSources): List<Violation> {
        val unitPath = sources.unitPath ?: return emptyList()
        val rootPackage = sources.layout.rootPackage
        if (rootPackage.isEmpty()) return emptyList()
        val segments = rootPackage.split('.')
        val candidates = UnitSources.unitNameCandidates(unitPath)
        if (candidates.any { candidate -> segments.takeLast(candidate.size) == candidate }) return emptyList()
        val pathSegments = unitPath.split(':').filter(String::isNotEmpty)
        val expected = pathSegments.drop(1).ifEmpty { pathSegments }.joinToString(".") { segment ->
            segment.replace("-", "")
        }
        val message = "Name the root package $rootPackage after the unit $unitPath: it ends with $expected, the unit " +
            "path in the project's convention."
        val violation = sources.createExternalViolation(
            RULE,
            BUILD_FILE,
            rootPackage,
            message
        )
        return listOf(violation)
    }

    private companion object {
        const val RULE = "RootPackagePath"
        const val BUILD_FILE = "build.gradle.kts"
    }
}
