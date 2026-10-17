package ru.astrainteractive.klibs.rules.konsist.check.placement

import ru.astrainteractive.klibs.rules.konsist.check.ArchitectureCheck
import ru.astrainteractive.klibs.rules.konsist.check.model.UnitSources
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import ru.astrainteractive.klibs.rules.konsist.check.model.packageName
import ru.astrainteractive.klibs.rules.konsist.layout.Kind
import ru.astrainteractive.klibs.rules.konsist.layout.UnitKind

/**
 * Reports a package created inside a unit with a word from the forbidden list of the package-layout rule. The
 * unit's root package is not judged: it follows the unit path and may keep a project's word (`…data.dao`).
 * `table` and `entity` are allowed only in a technology's implementation.
 */
internal class ForbiddenPackageName : ArchitectureCheck {
    override val unitKinds: Set<UnitKind> = UnitKind.entries.toSet()

    private fun isTechnologyWord(segment: String): Boolean {
        return Kind.fromSegment(segment) == Kind.IMPLEMENTATION
    }

    private fun findForbidden(sources: UnitSources, segments: List<String>): List<String> {
        return segments.filterIndexed { index, segment ->
            if (segment in FORBIDDEN) return@filterIndexed true
            if (segment !in TECHNOLOGY_ONLY) return@filterIndexed false
            val isInTechnology = sources.kind == UnitKind.TECHNOLOGY
            val followsTechnology = segments.take(index).any { previous -> isTechnologyWord(previous) }
            !isInTechnology && !followsTechnology
        }
    }

    override fun check(sources: UnitSources): List<Violation> {
        return sources.mainFiles
            .distinctBy { file -> file.packageName }
            .mapNotNull { file ->
                val segments = sources.layout.segmentsBelowRoot(file.packageName) ?: return@mapNotNull null
                val forbidden = findForbidden(sources, segments)
                if (forbidden.isEmpty()) return@mapNotNull null
                val message = "Rename the package segments ${forbidden.joinToString { segment -> "'$segment'" }}: " +
                    "a package is named for a concept or a kind, never a layer or a filler word."
                Violation(RULE, sources.relativePath(file), file.packageName, message)
            }
    }

    private companion object {
        const val RULE = "ForbiddenPackageName"

        @JvmField
        val TECHNOLOGY_ONLY = setOf("table", "entity")

        @JvmField
        val FORBIDDEN = setOf(
            "adapter",
            "aggregate",
            "application",
            "common",
            "controller",
            "dao",
            "domain",
            "dto",
            "entrypoint",
            "enum",
            "exception",
            "factory",
            "helpers",
            "impl",
            "in",
            "inbound",
            "infrastructure",
            "internal",
            "misc",
            "out",
            "outbound",
            "port",
            "presentation",
            "repository",
            "shared",
            "spi",
            "type",
            "ui",
            "validation",
            "valueobject",
        )
    }
}
