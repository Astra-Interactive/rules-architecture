package ru.astrainteractive.klibs.rules.konsist.check.declaration

import com.lemonappdev.konsist.api.KoModifier
import ru.astrainteractive.klibs.rules.konsist.check.ArchitectureCheck
import ru.astrainteractive.klibs.rules.konsist.check.model.UnitSources
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import ru.astrainteractive.klibs.rules.konsist.check.model.topLevelClasses
import ru.astrainteractive.klibs.rules.konsist.layout.Kind
import ru.astrainteractive.klibs.rules.konsist.layout.UnitKind

/**
 * Reports a public top-level class in a technology package (`exposed`, `memory`, `file`, …) or directly in a
 * concept of a technology unit: the unit's DI module constructs it, so it is `internal`. Stubs stay public,
 * because other units need them; a published library is not judged.
 */
internal class ImplementationVisibility : ArchitectureCheck {
    override val unitKinds: Set<UnitKind> = UnitKind.entries.toSet() - UnitKind.LIBRARY

    override fun check(sources: UnitSources): List<Violation> {
        return sources.mainFiles
            .filter { file ->
                val placement = sources.placementOf(file)
                placement?.kind == Kind.IMPLEMENTATION && placement.kindWord != STUB
            }
            .flatMap { file ->
                file.topLevelClasses()
                    .filterNot { declaration -> NON_PUBLIC.any { modifier -> declaration.hasModifier(modifier) } }
                    .map { declaration ->
                        val message = "Make ${declaration.name} internal: the DI module of the unit constructs it."
                        Violation(
                            RULE,
                            sources.relativePath(file),
                            declaration.name,
                            message
                        )
                    }
            }
    }

    private companion object {
        const val RULE = "ImplementationVisibility"
        const val STUB = "stub"

        @JvmField
        val NON_PUBLIC = listOf(
            KoModifier.INTERNAL,
            KoModifier.PRIVATE,
            KoModifier.PROTECTED
        )
    }
}
