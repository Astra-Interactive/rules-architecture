package ru.astrainteractive.klibs.rules.konsist.check.declaration

import com.lemonappdev.konsist.api.declaration.KoClassDeclaration
import com.lemonappdev.konsist.api.provider.KoNameProvider
import ru.astrainteractive.klibs.rules.konsist.check.ArchitectureCheck
import ru.astrainteractive.klibs.rules.konsist.check.model.UnitSources
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import ru.astrainteractive.klibs.rules.konsist.layout.Kind
import ru.astrainteractive.klibs.rules.konsist.layout.UnitKind

/**
 * Reports a property of a class outside `di` initialised with `X(…)` or `by lazy { X(…) }`, where `X` is a class of
 * the unit whose primary constructor takes something: a dependency is constructed in the DI module and passed in.
 */
internal class FieldDependencyConstruction : ArchitectureCheck {
    override val unitKinds: Set<UnitKind> = UnitKind.entries.toSet()

    private fun constructedName(propertyText: String): String? {
        val match = DIRECT_CONSTRUCTION.find(propertyText) ?: LAZY_CONSTRUCTION.find(propertyText) ?: return null
        return match.groupValues[1]
    }

    private fun dependencyClassNames(sources: UnitSources): Set<String> {
        return sources.mainFiles
            .flatMap { file -> file.classes(includeNested = false, includeLocal = false) }
            .filter { declaration -> declaration.primaryConstructor?.parameters.orEmpty().isNotEmpty() }
            .map(KoNameProvider::name)
            .toSet()
    }

    private fun constructedDependencies(declaration: KoClassDeclaration, dependencies: Set<String>): List<String> {
        return declaration.properties(includeNested = false)
            .mapNotNull { property -> constructedName(property.text) }
            .filter { name -> name in dependencies }
    }

    override fun check(sources: UnitSources): List<Violation> {
        val dependencies = dependencyClassNames(sources)
        return sources.mainFiles
            .filterNot { file -> sources.placementOf(file)?.kind == Kind.DI }
            .flatMap { file ->
                file.classes(includeNested = true, includeLocal = false).flatMap { declaration ->
                    constructedDependencies(declaration, dependencies).map { name ->
                        val message = "${declaration.name} constructs $name in a field: construct it in the DI " +
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
        val DIRECT_CONSTRUCTION = Regex("""=\s*([A-Z]\w*)\s*\(""")

        @JvmField
        val LAZY_CONSTRUCTION = Regex("""by\s+lazy\s*\{\s*([A-Z]\w*)\s*\(""")
    }
}
