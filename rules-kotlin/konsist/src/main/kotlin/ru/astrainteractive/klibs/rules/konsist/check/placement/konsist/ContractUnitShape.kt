package ru.astrainteractive.klibs.rules.konsist.check.placement.konsist

import com.lemonappdev.konsist.api.KoModifier
import com.lemonappdev.konsist.api.declaration.KoFileDeclaration
import com.lemonappdev.konsist.api.provider.KoNameProvider
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

    private fun unserializableTypes(file: KoFileDeclaration): List<String> {
        val classes = file.topLevelClasses()
            .filterNot { declaration -> declaration.hasModifier(KoModifier.ENUM) }
            .filterNot { declaration -> declaration.hasAnnotationWithName(SERIALIZABLE) }
            .map(KoNameProvider::name)
        val interfaces = file.interfaces(includeNested = false)
            .filterNot { declaration -> declaration.hasAnnotationWithName(SERIALIZABLE) }
            .map(KoNameProvider::name)
        return classes + interfaces
    }

    private fun functionsWithBody(file: KoFileDeclaration): List<String> {
        val topLevel = file.functions(includeNested = false, includeLocal = false)
        val inClasses = file.classes(includeNested = true, includeLocal = false)
            .flatMap { declaration -> declaration.functions(includeNested = false, includeLocal = false) }
        return topLevel.plus(inClasses)
            .filter { function -> function.hasBlockBody || function.hasExpressionBody }
            .map(KoNameProvider::name)
    }

    override fun check(sources: UnitSources): List<Violation> {
        return sources.mainFiles.flatMap { file ->
            val path = sources.relativePath(file)
            val types = unserializableTypes(file).map { name ->
                Violation(
                    RULE,
                    path,
                    name,
                    "Annotate $name @Serializable or move it out: a contract unit holds only wire types."
                )
            }
            val functions = functionsWithBody(file).map { name ->
                Violation(
                    RULE,
                    path,
                    name,
                    "Move $name out of the contract unit: a wire type carries no behaviour; map it at each side."
                )
            }
            types + functions
        }
    }

    private companion object {
        const val RULE = "ContractUnitShape"
        const val SERIALIZABLE = "Serializable"
    }
}
