package ru.astrainteractive.klibs.rules.konsist.check.placement.konsist

import com.lemonappdev.konsist.api.declaration.KoFileDeclaration
import ru.astrainteractive.klibs.rules.konsist.check.konsist.ArchitectureCheck
import ru.astrainteractive.klibs.rules.konsist.check.konsist.UnitSources
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import ru.astrainteractive.klibs.rules.konsist.check.model.util.packageName
import ru.astrainteractive.klibs.rules.konsist.layout.model.UnitKind

/**
 * Reports a package created inside a unit whose name is plural: every package name is singular, except an
 * inherently plural concept (`news`, `settings`, and the project's own words). A word is taken as plural when it
 * ends in `s` but not in `ss`, `us`, `is`, `as` or `ics` (`class`, `status`, `analysis`, `canvas`, `analytics`).
 * The unit's root package is not judged.
 */
internal class PluralPackageName : ArchitectureCheck {
    override val unitKinds: Set<UnitKind> = UnitKind.entries.toSet()

    private fun isPlural(sources: UnitSources, segment: String): Boolean {
        if (!segment.endsWith(PLURAL_ENDING) || segment in INHERENTLY_PLURAL) return false
        if (segment in sources.layout.vocabulary.pluralConcepts) return false
        return SINGULAR_ENDINGS.none(segment::endsWith)
    }

    override fun check(sources: UnitSources): List<Violation> {
        return sources.mainFiles
            .distinctBy(KoFileDeclaration::packageName)
            .mapNotNull { file ->
                val segments = sources.layout.segmentsBelowRoot(file.packageName) ?: return@mapNotNull null
                val plural = segments.filter { segment -> isPlural(sources, segment) }
                if (plural.isEmpty()) return@mapNotNull null
                val message = "Rename the package segments ${plural.joinToString { segment -> "'$segment'" }} to " +
                    "the singular: a package is named like one concept or one kind."
                sources.createViolation(
                    RULE,
                    file,
                    file.packageName,
                    message
                )
            }
    }

    private companion object {
        const val RULE = "PluralPackageName"
        const val PLURAL_ENDING = "s"

        @JvmField
        val SINGULAR_ENDINGS = listOf(
            "ss",
            "us",
            "is",
            "as",
            "ics"
        )

        @JvmField
        val INHERENTLY_PLURAL = setOf(
            "news",
            "series",
            "settings",
            "species"
        )
    }
}
