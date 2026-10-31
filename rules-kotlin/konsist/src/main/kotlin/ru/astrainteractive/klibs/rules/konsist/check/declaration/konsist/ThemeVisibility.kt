package ru.astrainteractive.klibs.rules.konsist.check.declaration.konsist

import com.lemonappdev.konsist.api.KoModifier
import ru.astrainteractive.klibs.rules.konsist.check.konsist.ArchitectureCheck
import ru.astrainteractive.klibs.rules.konsist.check.konsist.UnitSources
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import ru.astrainteractive.klibs.rules.konsist.layout.model.Kind
import ru.astrainteractive.klibs.rules.konsist.layout.model.UnitKind

/**
 * Reports a `theme` object of the wrong visibility: it is `internal` in a feature unit, and public in the
 * design-system unit (`core/ui`), whose theme other units read. A published library is not judged.
 */
internal class ThemeVisibility : ArchitectureCheck {
    override val unitKinds: Set<UnitKind> = UnitKind.entries.toSet() - UnitKind.LIBRARY

    private fun isDesignSystem(sources: UnitSources): Boolean {
        val segments = sources.unitPath.orEmpty().split(':')
        return CORE in segments && segments.last() == DESIGN_SYSTEM_NAME
    }

    override fun check(sources: UnitSources): List<Violation> {
        val isDesignSystem = isDesignSystem(sources)
        return sources.mainFiles
            .filter { file -> sources.placementOf(file)?.kind == Kind.THEME }
            .flatMap { file ->
                file.objects(includeNested = false).mapNotNull { declaration ->
                    val isHidden = HIDDEN.any(declaration::hasModifier)
                    val isInternal = declaration.hasModifier(KoModifier.INTERNAL)
                    val message = when {
                        isDesignSystem && isHidden ->
                            "Make ${declaration.name} public: the design-system unit's theme is read by other units."
                        !isDesignSystem && !isInternal ->
                            "Make ${declaration.name} internal: a feature's theme values are shared by its own " +
                                "composables only."
                        else -> null
                    } ?: return@mapNotNull null
                    sources.createViolation(
                        RULE,
                        file,
                        declaration,
                        declaration.name,
                        message
                    )
                }
            }
    }

    private companion object {
        const val RULE = "ThemeVisibility"
        const val CORE = "core"
        const val DESIGN_SYSTEM_NAME = "ui"

        @JvmField
        val HIDDEN = listOf(KoModifier.INTERNAL, KoModifier.PRIVATE)
    }
}
