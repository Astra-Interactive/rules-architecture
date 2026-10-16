package ru.astrainteractive.klibs.rules.konsist.check.declaration

import ru.astrainteractive.klibs.rules.konsist.check.ArchitectureCheck
import ru.astrainteractive.klibs.rules.konsist.check.model.UnitSources
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import ru.astrainteractive.klibs.rules.konsist.check.model.topLevelClasses
import ru.astrainteractive.klibs.rules.konsist.layout.Layer
import ru.astrainteractive.klibs.rules.konsist.layout.UnitKind

/**
 * Reports a public class or object in an entry-point role: the entry-point unit constructs the context's DI
 * module, never its commands, listeners or routes.
 */
internal class EntryPointVisibility : ArchitectureCheck {
    override val unitKinds: Set<UnitKind> = setOf(UnitKind.CONTEXT)

    override fun check(sources: UnitSources): List<Violation> = sources.mainFiles
        .filter { file -> sources.placementOf(file)?.layer == Layer.ENTRY_POINT }
        .flatMap { file ->
            val classes = file.topLevelClasses()
                .filterNot { declaration -> declaration.hasInternalModifier || declaration.hasPrivateModifier }
                .map { declaration -> declaration.name }
            val objects = file.objects(includeNested = false)
                .filterNot { declaration -> declaration.hasInternalModifier || declaration.hasPrivateModifier }
                .map { declaration -> declaration.name }
            (classes + objects).map { name ->
                val message = "Make $name internal: the entry-point unit constructs the context's DI module, " +
                    "not its entry points."
                Violation(RULE, sources.relativePath(file), name, message)
            }
        }

    private companion object {
        const val RULE = "EntryPointVisibility"
    }
}
