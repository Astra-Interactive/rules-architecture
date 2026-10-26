package ru.astrainteractive.klibs.rules.konsist.check.declaration.konsist

import ru.astrainteractive.klibs.rules.konsist.check.konsist.ArchitectureCheck
import ru.astrainteractive.klibs.rules.konsist.check.konsist.UnitSources
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import ru.astrainteractive.klibs.rules.konsist.check.model.util.topLevelClasses
import ru.astrainteractive.klibs.rules.konsist.layout.model.UnitKind

/**
 * Reports a public class or object in a kind the platform invokes (`command`, `event`, `expansion`, `route`,
 * `job`, `service`, `warmup`, `migration`, `error`): other units construct the unit's DI module, never its entry
 * points. UI state holders are left out, because another unit's screen calls them, and so is a published
 * library, whose public API may live in these kinds.
 */
internal class EntryPointVisibility : ArchitectureCheck {
    override val unitKinds: Set<UnitKind> = UnitKind.entries.toSet() - UnitKind.LIBRARY

    override fun check(sources: UnitSources): List<Violation> {
        return sources.mainFiles
            .filter { file -> sources.placementOf(file)?.kindWord in PLATFORM_INVOKED_KINDS }
            .flatMap { file ->
                val classes = file.topLevelClasses()
                    .filterNot { declaration -> declaration.hasInternalModifier || declaration.hasPrivateModifier }
                val objects = file.objects(includeNested = false)
                    .filterNot { declaration -> declaration.hasInternalModifier || declaration.hasPrivateModifier }
                classes.plus(objects).map { declaration ->
                    val message = "Make ${declaration.name} internal: other units construct the unit's DI module, " +
                        "not its entry points."
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
        const val RULE = "EntryPointVisibility"

        @JvmField
        val PLATFORM_INVOKED_KINDS = setOf(
            "command",
            "error",
            "event",
            "expansion",
            "gui",
            "job",
            "menu",
            "migration",
            "route",
            "service",
            "warmup",
        )
    }
}
