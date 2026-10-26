package ru.astrainteractive.klibs.rules.konsist.check.declaration.konsist

import com.lemonappdev.konsist.api.declaration.KoBaseDeclaration
import com.lemonappdev.konsist.api.declaration.KoFileDeclaration
import ru.astrainteractive.klibs.rules.konsist.check.konsist.ArchitectureCheck
import ru.astrainteractive.klibs.rules.konsist.check.konsist.UnitSources
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import ru.astrainteractive.klibs.rules.konsist.check.model.util.topLevelTypeNames
import ru.astrainteractive.klibs.rules.konsist.layout.model.Kind
import ru.astrainteractive.klibs.rules.konsist.layout.model.UnitKind

/**
 * Reports the state type of a screen or a menu (`<Screen>State`, a name of at least three letters before
 * `State`) nested inside another type or declared outside a `model` package: the view reads it, so it is its own
 * file in `model/`.
 */
internal class StateTypePlacement : ArchitectureCheck {
    override val unitKinds: Set<UnitKind> = UnitKind.entries.toSet()

    private fun stateDeclarations(file: KoFileDeclaration, includeNested: Boolean): List<StateDeclaration> {
        val classes = file.classes(includeNested = includeNested, includeLocal = false)
            .map { declaration -> StateDeclaration(declaration, declaration.name) }
        val interfaces = file.interfaces(includeNested = includeNested)
            .map { declaration -> StateDeclaration(declaration, declaration.name) }
        val objects = file.objects(includeNested = includeNested)
            .map { declaration -> StateDeclaration(declaration, declaration.name) }
        return classes + interfaces + objects
    }

    private fun nestedStateDeclarations(file: KoFileDeclaration): List<StateDeclaration> {
        val topLevelNames = file.topLevelTypeNames().toSet()
        return stateDeclarations(file, includeNested = true)
            .filterNot { declaration -> declaration.name in topLevelNames }
    }

    override fun check(sources: UnitSources): List<Violation> {
        return sources.mainFiles.flatMap { file ->
            val nested = nestedStateDeclarations(file)
                .filter { declaration -> STATE_NAME.matches(declaration.name) }
                .map { declaration ->
                    val message = "Move ${declaration.name} to its own file in model/: a screen state is never " +
                        "nested in its view model or state holder."
                    sources.createViolation(
                        RULE,
                        file,
                        declaration.declaration,
                        declaration.name,
                        message
                    )
                }
            val isModel = sources.placementOf(file)?.kind == Kind.MODEL
            val misplaced = stateDeclarations(file, includeNested = false)
                .filter { declaration -> STATE_NAME.matches(declaration.name) && !isModel }
                .map { declaration ->
                    val message = "Move ${declaration.name} to the model package of its level: the view reads the " +
                        "screen state."
                    sources.createViolation(
                        RULE,
                        file,
                        declaration.declaration,
                        declaration.name,
                        message
                    )
                }
            nested + misplaced
        }
    }

    private class StateDeclaration(val declaration: KoBaseDeclaration, val name: String)

    private companion object {
        const val RULE = "StateTypePlacement"

        @JvmField
        val STATE_NAME = Regex("(?!Mutable)[A-Z][A-Za-z0-9]{2,}State")
    }
}
