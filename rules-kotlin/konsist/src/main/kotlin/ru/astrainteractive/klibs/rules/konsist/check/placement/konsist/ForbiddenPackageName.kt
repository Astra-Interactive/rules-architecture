package ru.astrainteractive.klibs.rules.konsist.check.placement.konsist

import com.lemonappdev.konsist.api.declaration.KoFileDeclaration
import ru.astrainteractive.klibs.rules.konsist.check.konsist.ArchitectureCheck
import ru.astrainteractive.klibs.rules.konsist.check.konsist.UnitSources
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import ru.astrainteractive.klibs.rules.konsist.check.model.util.packageName
import ru.astrainteractive.klibs.rules.konsist.layout.model.Kind
import ru.astrainteractive.klibs.rules.konsist.layout.model.UnitKind

/**
 * Reports a package created inside a unit with a word from the forbidden list of the package-layout rule. The
 * unit's root package is not judged: it follows the unit path and may keep a project's word (`…data.dao`).
 * `table`, `entity` and `database` are allowed only in a technology's implementation: in a technology unit or
 * below a technology package.
 */
internal class ForbiddenPackageName : ArchitectureCheck {
    override val unitKinds: Set<UnitKind> = UnitKind.entries.toSet()

    private fun isTechnologyWord(sources: UnitSources, segment: String): Boolean {
        return Kind.fromSegment(segment, sources.layout.vocabulary) == Kind.IMPLEMENTATION
    }

    private fun findForbidden(sources: UnitSources, segments: List<String>): List<String> {
        return segments.filterIndexed { index, segment ->
            if (segment in FORBIDDEN) return@filterIndexed true
            if (segment !in TECHNOLOGY_ONLY) return@filterIndexed false
            val isInTechnology = sources.kind == UnitKind.TECHNOLOGY
            val followsTechnology = segments.take(index).any { previous -> isTechnologyWord(sources, previous) }
            !isInTechnology && !followsTechnology
        }
    }

    override fun check(sources: UnitSources): List<Violation> {
        return sources.mainFiles
            .distinctBy(KoFileDeclaration::packageName)
            .mapNotNull { file ->
                val segments = sources.layout.segmentsBelowRoot(file.packageName) ?: return@mapNotNull null
                val forbidden = findForbidden(sources, segments)
                if (forbidden.isEmpty()) return@mapNotNull null
                val message = "Rename the package segments ${forbidden.joinToString { segment -> "'$segment'" }}: " +
                    "a package is named for a concept or a kind, never a layer or a filler word."
                sources.createViolation(
                    RULE,
                    file,
                    file.packageName,
                    message
                )
            }
    }

    private companion object {
        const val RULE = "ForbiddenPackageName"

        @JvmField
        val TECHNOLOGY_ONLY = setOf(
            "database",
            "entity",
            "table"
        )

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
