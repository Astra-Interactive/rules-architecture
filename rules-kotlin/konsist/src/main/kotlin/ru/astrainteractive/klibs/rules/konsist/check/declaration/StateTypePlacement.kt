package ru.astrainteractive.klibs.rules.konsist.check.declaration

import com.lemonappdev.konsist.api.declaration.KoFileDeclaration
import com.lemonappdev.konsist.api.provider.KoNameProvider
import ru.astrainteractive.klibs.rules.konsist.check.ArchitectureCheck
import ru.astrainteractive.klibs.rules.konsist.check.model.UnitSources
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import ru.astrainteractive.klibs.rules.konsist.check.model.topLevelTypeNames
import ru.astrainteractive.klibs.rules.konsist.layout.Kind
import ru.astrainteractive.klibs.rules.konsist.layout.UnitKind

/**
 * Reports the state type of a screen or a menu (`<Screen>State`, a name of at least three letters before
 * `State`) nested inside another type or declared outside a `model` package: the view reads it, so it is its own
 * file in `model/`.
 */
internal class StateTypePlacement : ArchitectureCheck {
    override val unitKinds: Set<UnitKind> = UnitKind.entries.toSet()

    private fun nestedStateNames(file: KoFileDeclaration): List<String> {
        val nested = file.classes(includeNested = true, includeLocal = false).map(KoNameProvider::name) +
            file.interfaces(includeNested = true).map(KoNameProvider::name) +
            file.objects(includeNested = true).map(KoNameProvider::name)
        return nested - file.topLevelTypeNames().toSet()
    }

    override fun check(sources: UnitSources): List<Violation> {
        return sources.mainFiles.flatMap { file ->
            val path = sources.relativePath(file)
            val nested = nestedStateNames(file)
                .filter(STATE_NAME::matches)
                .map { name ->
                    val message = "Move $name to its own file in model/: a screen state is never nested in its view " +
                        "model or state holder."
                    Violation(
                        RULE,
                        path,
                        name,
                        message
                    )
                }
            val isModel = sources.placementOf(file)?.kind == Kind.MODEL
            val misplaced = file.topLevelTypeNames()
                .filter { name -> STATE_NAME.matches(name) && !isModel }
                .map { name ->
                    val message = "Move $name to the model package of its level: the view reads the screen state."
                    Violation(
                        RULE,
                        path,
                        name,
                        message
                    )
                }
            nested + misplaced
        }
    }

    private companion object {
        const val RULE = "StateTypePlacement"

        @JvmField
        val STATE_NAME = Regex("(?!Mutable)[A-Z][A-Za-z0-9]{2,}State")
    }
}
