package ru.astrainteractive.klibs.rules.konsist.check.declaration

import com.lemonappdev.konsist.api.declaration.KoClassDeclaration
import com.lemonappdev.konsist.api.declaration.KoObjectDeclaration
import ru.astrainteractive.klibs.rules.konsist.check.ArchitectureCheck
import ru.astrainteractive.klibs.rules.konsist.check.model.UnitSources
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import ru.astrainteractive.klibs.rules.konsist.check.model.topLevelClasses
import ru.astrainteractive.klibs.rules.konsist.layout.UnitKind

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
                    .map(KoClassDeclaration::name)
                val objects = file.objects(includeNested = false)
                    .filterNot { declaration -> declaration.hasInternalModifier || declaration.hasPrivateModifier }
                    .map(KoObjectDeclaration::name)
                classes.plus(objects).map { name ->
                    val message = "Make $name internal: other units construct the unit's DI module, " +
                        "not its entry points."
                    Violation(
                        RULE,
                        sources.relativePath(file),
                        name,
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
