package ru.astrainteractive.klibs.rules.konsist.check.placement

import ru.astrainteractive.klibs.rules.konsist.check.ArchitectureCheck
import ru.astrainteractive.klibs.rules.konsist.check.model.UnitSources
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import ru.astrainteractive.klibs.rules.konsist.check.model.topLevelClasses
import ru.astrainteractive.klibs.rules.konsist.layout.Kind
import ru.astrainteractive.klibs.rules.konsist.layout.UnitKind

/**
 * Reports a `RootModule` outside the entry-point unit or outside its `di` package: the composition root lives in
 * `di/RootModule.kt` of the unit that starts the application.
 */
internal class RootModulePlacement : ArchitectureCheck {
    override val unitKinds: Set<UnitKind> = UnitKind.entries.toSet()

    override fun check(sources: UnitSources): List<Violation> {
        return sources.mainFiles.flatMap { file ->
            val isDi = sources.placementOf(file)?.kind == Kind.DI
            file.topLevelClasses()
                .filter { declaration -> declaration.name == ROOT_MODULE }
                .mapNotNull { declaration ->
                    val message = when {
                        sources.kind != UnitKind.ENTRY_POINT ->
                            "Move RootModule to the entry-point unit: the composition root lives next to the " +
                                "platform main class that creates it."
                        !isDi -> "Move RootModule to the di package of the entry-point unit."
                        else -> null
                    } ?: return@mapNotNull null
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
        const val RULE = "RootModulePlacement"
        const val ROOT_MODULE = "RootModule"
    }
}
