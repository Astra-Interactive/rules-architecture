package ru.astrainteractive.klibs.rules.konsist.check.declaration.konsist

import com.lemonappdev.konsist.api.KoModifier
import ru.astrainteractive.klibs.rules.konsist.check.konsist.ArchitectureCheck
import ru.astrainteractive.klibs.rules.konsist.check.konsist.UnitSources
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import ru.astrainteractive.klibs.rules.konsist.check.model.util.topLevelClasses
import ru.astrainteractive.klibs.rules.konsist.layout.model.Kind
import ru.astrainteractive.klibs.rules.konsist.layout.model.UnitKind

/**
 * Reports a public DI module that a file of its own unit constructs: it is a child in the unit's tree of DI
 * modules, so it is `internal`; only a DI module another unit constructs is public. A published library is not
 * judged.
 */
internal class DiModuleVisibility : ArchitectureCheck {
    override val unitKinds: Set<UnitKind> = UnitKind.entries.toSet() - UnitKind.LIBRARY

    override fun check(sources: UnitSources): List<Violation> {
        return sources.mainFiles
            .filter { file -> sources.placementOf(file)?.kind == Kind.DI }
            .flatMap { file ->
                file.topLevelClasses()
                    .filter { declaration -> declaration.name.endsWith(MODULE_SUFFIX) }
                    .filterNot { declaration -> NON_PUBLIC.any(declaration::hasModifier) }
                    .filter { declaration ->
                        val construction = Regex("""\b${Regex.escape(declaration.name)}\s*\(""")
                        sources.mainFiles
                            .filter { other -> other.path != file.path }
                            .any { other -> construction.containsMatchIn(other.text) }
                    }
                    .map { declaration ->
                        val message = "Make ${declaration.name} internal: a DI module of this unit constructs it, " +
                            "and only a DI module another unit constructs is public."
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
        const val RULE = "DiModuleVisibility"
        const val MODULE_SUFFIX = "Module"

        @JvmField
        val NON_PUBLIC = listOf(
            KoModifier.INTERNAL,
            KoModifier.PRIVATE,
            KoModifier.PROTECTED
        )
    }
}
