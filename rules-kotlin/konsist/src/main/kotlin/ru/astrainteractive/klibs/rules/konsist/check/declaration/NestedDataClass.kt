package ru.astrainteractive.klibs.rules.konsist.check.declaration

import com.lemonappdev.konsist.api.KoModifier
import com.lemonappdev.konsist.api.declaration.KoClassDeclaration
import com.lemonappdev.konsist.api.declaration.KoFileDeclaration
import ru.astrainteractive.klibs.rules.konsist.check.ArchitectureCheck
import ru.astrainteractive.klibs.rules.konsist.check.model.UnitSources
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import ru.astrainteractive.klibs.rules.konsist.check.model.topLevelClasses
import ru.astrainteractive.klibs.rules.konsist.check.model.typeNames
import ru.astrainteractive.klibs.rules.konsist.layout.UnitKind

/**
 * Reports a top-level data class that is the property type of exactly one class and is named in no file but
 * its own and that class's: it is nested in its owner. A data class with a supertype (the variant of a sealed
 * family) and the state of a screen or a menu (`*State`) are families of their own in `model`, never nested.
 */
internal class NestedDataClass : ArchitectureCheck {
    override val unitKinds: Set<UnitKind> = UnitKind.entries.toSet()

    private class DeclaredClass(val file: KoFileDeclaration, val declaration: KoClassDeclaration)

    private fun isFamilyOfItsOwn(declaration: KoClassDeclaration): Boolean {
        return declaration.hasParents() || declaration.name.endsWith(STATE_SUFFIX)
    }

    private fun propertyTypeNames(declaration: KoClassDeclaration): Set<String> {
        val constructorTypes = declaration.primaryConstructor?.parameters.orEmpty()
            .filter { parameter -> parameter.isVal || parameter.isVar }
            .flatMap { parameter -> parameter.type.typeNames }
        val bodyTypes = declaration.properties(includeNested = false)
            .flatMap { property -> property.type?.typeNames.orEmpty() }
        return constructorTypes.plus(bodyTypes).toSet()
    }

    private fun referencingFiles(name: String, files: List<KoFileDeclaration>): Set<String> {
        val word = Regex("\\b${Regex.escape(name)}\\b")
        return files.filter { file -> word.containsMatchIn(file.text) }.map(KoFileDeclaration::path).toSet()
    }

    private fun findOwner(
        dataClass: DeclaredClass,
        classes: List<DeclaredClass>,
        files: List<KoFileDeclaration>
    ): DeclaredClass? {
        val name = dataClass.declaration.name
        val owners = classes.filter { candidate ->
            candidate.declaration.name != name && name in propertyTypeNames(candidate.declaration)
        }
        val owner = owners.singleOrNull() ?: return null
        val allowedFiles = setOf(dataClass.file.path, owner.file.path)
        return owner.takeIf { allowedFiles.containsAll(referencingFiles(name, files)) }
    }

    override fun check(sources: UnitSources): List<Violation> {
        val classes = sources.mainFiles.flatMap { file ->
            file.classes(
                includeNested = true,
                includeLocal = false
            ).map { declaration -> DeclaredClass(file, declaration) }
        }
        return sources.mainFiles
            .flatMap { file -> file.topLevelClasses().map { declaration -> DeclaredClass(file, declaration) } }
            .filter { declared -> declared.declaration.hasModifier(KoModifier.DATA) }
            .filterNot { declared -> isFamilyOfItsOwn(declared.declaration) }
            .mapNotNull { dataClass ->
                val owner = findOwner(
                    dataClass,
                    classes,
                    sources.mainFiles
                ) ?: return@mapNotNull null
                val name = dataClass.declaration.name
                val message = "Nest $name in ${owner.declaration.name}: it is the property type of that class only."
                Violation(
                    RULE,
                    sources.relativePath(dataClass.file),
                    name,
                    message
                )
            }
    }

    private companion object {
        const val RULE = "NestedDataClass"
        const val STATE_SUFFIX = "State"
    }
}
