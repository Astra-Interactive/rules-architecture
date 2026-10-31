package ru.astrainteractive.klibs.rules.konsist.check.placement.konsist

import ru.astrainteractive.klibs.rules.konsist.check.konsist.ArchitectureCheck
import ru.astrainteractive.klibs.rules.konsist.check.konsist.UnitSources
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import ru.astrainteractive.klibs.rules.konsist.layout.model.UnitKind

/**
 * Reports a unit whose root package does not end with the unit's name taken from its path: `modules/jail` ends
 * at `jail`, `modules/event-bukkit` at `event`, `components/shop/payment/impl` at `payment.impl`. A unit of `core`
 * may end at `core` (`modules/core/api` and `modules/core/bukkit` at `…core`), and the last segment may be the
 * singular of a plural unit name (`modules/money-advancements` at `moneyadvancement`), since a package is singular.
 * The entry point, named for the application, and a published library are not judged.
 */
internal class RootPackagePath : ArchitectureCheck {
    override val unitKinds: Set<UnitKind> = UnitKind.entries.toSet() - UnitKind.ENTRY_POINT - UnitKind.LIBRARY

    private fun coreEndings(pathSegments: List<String>): List<List<String>> {
        val coreIndex = pathSegments.indexOf(CORE)
        if (coreIndex !in 0 until pathSegments.lastIndex) return emptyList()
        return (coreIndex downTo 0).map { start -> pathSegments.subList(start, coreIndex + 1) }
    }

    private fun singular(ending: List<String>): List<String>? {
        val last = ending.lastOrNull() ?: return null
        val singular = when {
            last.endsWith(IES) -> last.dropLast(IES.length) + Y
            last.endsWith(S) && !last.endsWith(SS) -> last.dropLast(S.length)
            else -> return null
        }
        return ending.dropLast(1) + singular
    }

    override fun check(sources: UnitSources): List<Violation> {
        val unitPath = sources.unitPath ?: return emptyList()
        val rootPackage = sources.layout.rootPackage
        if (rootPackage.isEmpty()) return emptyList()
        val segments = rootPackage.split('.')
        val pathSegments = unitPath.split(':').filter(String::isNotEmpty)
        val candidates = UnitSources.unitNameCandidates(unitPath)
        val endings = candidates + candidates.mapNotNull(::singular) + coreEndings(pathSegments)
        if (endings.any { ending -> segments.takeLast(ending.size) == ending }) return emptyList()
        val expected = coreEndings(pathSegments).firstOrNull()?.joinToString(".")
            ?: pathSegments.drop(1).ifEmpty { pathSegments }.joinToString(".") { segment -> segment.replace("-", "") }
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
        const val CORE = "core"
        const val IES = "ies"
        const val Y = "y"
        const val S = "s"
        const val SS = "ss"
    }
}
