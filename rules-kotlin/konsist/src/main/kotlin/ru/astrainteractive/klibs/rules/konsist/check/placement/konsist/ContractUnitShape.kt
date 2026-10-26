package ru.astrainteractive.klibs.rules.konsist.check.placement.konsist

import com.lemonappdev.konsist.api.KoModifier
import com.lemonappdev.konsist.api.declaration.KoFileDeclaration
import ru.astrainteractive.klibs.rules.konsist.check.konsist.ArchitectureCheck
import ru.astrainteractive.klibs.rules.konsist.check.konsist.UnitSources
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import ru.astrainteractive.klibs.rules.konsist.check.model.util.topLevelClasses
import ru.astrainteractive.klibs.rules.konsist.layout.model.UnitKind

/**
 * Checks a `<context>/contract` unit holds only wire types: every top-level class and interface is `@Serializable`
 * (objects and enums of route constants aside), and no function outside an object has a body.
 */
internal class ContractUnitShape : ArchitectureCheck {
    override val unitKinds: Set<UnitKind> = setOf(UnitKind.CONTRACT)

    private fun unserializableTypes(sources: UnitSources, file: KoFileDeclaration): List<Violation> {
        val classes = file.topLevelClasses()
            .filterNot { declaration -> declaration.hasModifier(KoModifier.ENUM) }
            .filterNot { declaration -> declaration.hasAnnotationWithName(SERIALIZABLE) }
            .map { declaration ->
                sources.createViolation(
                    RULE,
                    file,
                    declaration,
                    declaration.name,
                    "Annotate ${declaration.name} @Serializable or move it out: a contract unit holds only wire types."
                )
            }
        val interfaces = file.interfaces(includeNested = false)
            .filterNot { declaration -> declaration.hasAnnotationWithName(SERIALIZABLE) }
            .map { declaration ->
                sources.createViolation(
                    RULE,
                    file,
                    declaration,
                    declaration.name,
                    "Annotate ${declaration.name} @Serializable or move it out: a contract unit holds only wire types."
                )
            }
        return classes + interfaces
    }

    private fun functionsWithBody(sources: UnitSources, file: KoFileDeclaration): List<Violation> {
        val topLevel = file.functions(includeNested = false, includeLocal = false)
        val inClasses = file.classes(includeNested = true, includeLocal = false)
            .flatMap { declaration -> declaration.functions(includeNested = false, includeLocal = false) }
        return topLevel.plus(inClasses)
            .filter { function -> function.hasBlockBody || function.hasExpressionBody }
            .map { function ->
                sources.createViolation(
                    RULE,
                    file,
                    function,
                    function.name,
                    "Move ${function.name} out of the contract unit: a wire type carries no behaviour; map it at " +
                        "each side."
                )
            }
    }

    override fun check(sources: UnitSources): List<Violation> {
        return sources.mainFiles.flatMap { file ->
            unserializableTypes(sources, file) + functionsWithBody(sources, file)
        }
    }

    private companion object {
        const val RULE = "ContractUnitShape"
        const val SERIALIZABLE = "Serializable"
    }
}
