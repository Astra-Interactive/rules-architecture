package ru.astrainteractive.klibs.rules.konsist.check.declaration

import com.lemonappdev.konsist.api.KoModifier
import com.lemonappdev.konsist.api.declaration.KoClassDeclaration
import com.lemonappdev.konsist.api.declaration.KoFileDeclaration
import ru.astrainteractive.klibs.rules.konsist.check.ArchitectureCheck
import ru.astrainteractive.klibs.rules.konsist.check.model.UnitSources
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import ru.astrainteractive.klibs.rules.konsist.check.model.topLevelClasses
import ru.astrainteractive.klibs.rules.konsist.check.util.pathsNaming
import ru.astrainteractive.klibs.rules.konsist.check.util.propertyTypeNames
import ru.astrainteractive.klibs.rules.konsist.layout.UnitKind

/**
 * Reports a top-level enum class that is the property type of exactly one class and is named in no file but its
 * own and that class's: an enum used only through one type is nested in it.
 */
internal class NestedEnum : ArchitectureCheck {
    override val unitKinds: Set<UnitKind> = UnitKind.entries.toSet()

    private fun findOwner(enum: KoClassDeclaration, enumFile: KoFileDeclaration, sources: UnitSources): String? {
        val owners = sources.mainFiles.flatMap { file ->
            file.classes(includeNested = true, includeLocal = false)
                .filter { candidate -> candidate.name != enum.name && enum.name in candidate.propertyTypeNames() }
                .map { candidate -> DeclaredOwner(candidate.name, file.path) }
        }
        val owner = owners.singleOrNull() ?: return null
        val allowedFiles = setOf(enumFile.path, owner.path)
        return owner.name.takeIf { allowedFiles.containsAll(sources.mainFiles.pathsNaming(enum.name)) }
    }

    override fun check(sources: UnitSources): List<Violation> {
        return sources.mainFiles.flatMap { file ->
            file.topLevelClasses()
                .filter { declaration -> declaration.hasModifier(KoModifier.ENUM) }
                .mapNotNull { enum ->
                    val owner = findOwner(
                        enum,
                        file,
                        sources
                    ) ?: return@mapNotNull null
                    val message = "Nest ${enum.name} in $owner: an enum used only through one type is nested in it."
                    Violation(
                        RULE,
                        sources.relativePath(file),
                        enum.name,
                        message
                    )
                }
        }
    }

    private class DeclaredOwner(val name: String, val path: String)

    private companion object {
        const val RULE = "NestedEnum"
    }
}
