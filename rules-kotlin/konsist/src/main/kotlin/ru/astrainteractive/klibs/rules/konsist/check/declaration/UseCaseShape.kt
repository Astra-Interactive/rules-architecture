package ru.astrainteractive.klibs.rules.konsist.check.declaration

import com.lemonappdev.konsist.api.declaration.KoClassDeclaration
import ru.astrainteractive.klibs.rules.konsist.check.ArchitectureCheck
import ru.astrainteractive.klibs.rules.konsist.check.model.UnitSources
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import ru.astrainteractive.klibs.rules.konsist.check.model.isUseCase
import ru.astrainteractive.klibs.rules.konsist.check.model.simpleName
import ru.astrainteractive.klibs.rules.konsist.check.model.topLevelClasses
import ru.astrainteractive.klibs.rules.konsist.layout.Layer
import ru.astrainteractive.klibs.rules.konsist.layout.UnitKind

/**
 * Reports a use case with more than one public entry, and a use case that takes another use case: an entry
 * point that needs two actions calls both.
 */
internal class UseCaseShape : ArchitectureCheck {
    override val unitKinds: Set<UnitKind> = setOf(UnitKind.CONTEXT)

    private fun findEntryViolation(useCase: KoClassDeclaration): String? {
        val entries = useCase.functions(includeNested = false, includeLocal = false)
            .filterNot { function -> function.hasPrivateModifier }
            .map { function -> function.name }
        if (entries.size <= 1) return null
        return "${useCase.name} has the non-private functions ${entries.joinToString()}: a use case has one " +
            "public entry, operator fun invoke."
    }

    private fun findCalledUseCases(useCase: KoClassDeclaration, useCaseNames: Set<String>): List<String> =
        useCase.primaryConstructor?.parameters.orEmpty()
            .map { parameter -> parameter.type.simpleName }
            .filter { typeName -> typeName in useCaseNames && typeName != useCase.name }

    override fun check(sources: UnitSources): List<Violation> {
        val rootFiles = sources.mainFiles.filter { file -> sources.placementOf(file)?.layer == Layer.USE_CASE }
        val useCaseNames = rootFiles
            .flatMap { file -> file.topLevelClasses() }
            .filter { declaration -> declaration.isUseCase() }
            .map { declaration -> declaration.name }
            .toSet()
        return rootFiles.flatMap { file ->
            val path = sources.relativePath(file)
            file.topLevelClasses()
                .filter { declaration -> declaration.isUseCase() }
                .flatMap { useCase ->
                    val entryViolation = findEntryViolation(useCase)
                        ?.let { message -> Violation(RULE, path, useCase.name, message) }
                    val callViolations = findCalledUseCases(useCase, useCaseNames).map { calledName ->
                        val message = "${useCase.name} takes the use case $calledName: a use case never calls " +
                            "another use case; the entry point calls both."
                        Violation(RULE, path, "${useCase.name} -> $calledName", message)
                    }
                    listOfNotNull(entryViolation) + callViolations
                }
        }
    }

    private companion object {
        const val RULE = "UseCaseShape"
    }
}
