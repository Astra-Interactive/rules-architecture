package ru.astrainteractive.klibs.rules.konsist.check.declaration.konsist

import com.lemonappdev.konsist.api.KoModifier
import com.lemonappdev.konsist.api.declaration.KoClassDeclaration
import com.lemonappdev.konsist.api.declaration.KoFileDeclaration
import com.lemonappdev.konsist.api.provider.KoNameProvider
import ru.astrainteractive.klibs.rules.konsist.check.konsist.ArchitectureCheck
import ru.astrainteractive.klibs.rules.konsist.check.konsist.UnitSources
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import ru.astrainteractive.klibs.rules.konsist.check.model.util.topLevelClasses
import ru.astrainteractive.klibs.rules.konsist.layout.model.Kind
import ru.astrainteractive.klibs.rules.konsist.layout.model.UnitKind

/**
 * Reports construction of a concrete behaviour class of the unit outside `di`, in a property, constructor body or
 * method, with zero-argument constructors included. Models and value holders are not dependencies.
 */
internal class FieldDependencyConstruction : ArchitectureCheck {
    override val unitKinds: Set<UnitKind> = UnitKind.entries.toSet()

    private fun dependencyClassNames(sources: UnitSources): Set<String> {
        return sources.mainFiles
            .filter { file -> sources.placementOf(file)?.kind != Kind.MODEL }
            .flatMap(KoFileDeclaration::topLevelClasses)
            .filterNot { declaration -> declaration.hasModifier(KoModifier.DATA) }
            .filterNot { declaration -> declaration.hasModifier(KoModifier.ENUM) }
            .map(KoNameProvider::name)
            .toSet()
    }

    private fun constructedDependencies(declaration: KoClassDeclaration, dependencies: Set<String>): List<String> {
        return CONSTRUCTION.findAll(declaration.text.substringAfter('{', missingDelimiterValue = ""))
            .map { match -> match.groupValues[1] }
            .filter { name -> name != declaration.name && name in dependencies }
            .toList()
    }

    override fun check(sources: UnitSources): List<Violation> {
        val dependencies = dependencyClassNames(sources)
        return sources.mainFiles
            .filterNot { file -> sources.placementOf(file)?.kind == Kind.DI }
            .flatMap { file ->
                file.classes(includeNested = true, includeLocal = false).flatMap { declaration ->
                    constructedDependencies(declaration, dependencies).map { name ->
                        val message = "${declaration.name} constructs $name outside di: construct it in the DI " +
                            "module and pass it in through the constructor."
                        Violation(
                            RULE,
                            sources.relativePath(file),
                            "${declaration.name}.$name",
                            message
                        )
                    }
                }
            }
    }

    private companion object {
        const val RULE = "FieldDependencyConstruction"

        @JvmField
        val CONSTRUCTION = Regex("""\b([A-Z]\w*)\s*\(""")
    }
}
