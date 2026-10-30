package ru.astrainteractive.klibs.rules.konsist.check.declaration.konsist

import com.lemonappdev.konsist.api.KoModifier
import ru.astrainteractive.klibs.rules.konsist.check.konsist.ArchitectureCheck
import ru.astrainteractive.klibs.rules.konsist.check.konsist.UnitSources
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import ru.astrainteractive.klibs.rules.konsist.check.model.util.topLevelClasses
import ru.astrainteractive.klibs.rules.konsist.layout.model.Kind
import ru.astrainteractive.klibs.rules.konsist.layout.model.UnitKind

/**
 * Reports a public top-level class in a technology package (`exposed`, `file`, …) or directly in a concept of a
 * technology unit: the unit's DI module constructs it, so it is `internal`. Classes other units use stay public: a
 * stub, an in-memory implementation in `memory` that any unit's tests and demos use, and a generic class in `core`
 * (`core/cache/cache4k/RefreshingCache`). A published library is not judged.
 */
internal class ImplementationVisibility : ArchitectureCheck {
    override val unitKinds: Set<UnitKind> = UnitKind.entries.toSet() - UnitKind.LIBRARY

    override fun check(sources: UnitSources): List<Violation> {
        val isCore = sources.unitPath.orEmpty().split(':').contains(CORE)
        if (isCore) return emptyList()
        return sources.mainFiles
            .filter { file ->
                val placement = sources.placementOf(file)
                placement?.kind == Kind.IMPLEMENTATION && placement.kindWord !in SHARED_KIND_WORDS
            }
            .flatMap { file ->
                file.topLevelClasses()
                    .filterNot { declaration -> NON_PUBLIC.any(declaration::hasModifier) }
                    .map { declaration ->
                        val message = "Make ${declaration.name} internal: the DI module of the unit constructs it."
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
        const val RULE = "ImplementationVisibility"
        const val CORE = "core"

        @JvmField
        val SHARED_KIND_WORDS = setOf("stub", "memory")

        @JvmField
        val NON_PUBLIC = listOf(
            KoModifier.INTERNAL,
            KoModifier.PRIVATE,
            KoModifier.PROTECTED
        )
    }
}
