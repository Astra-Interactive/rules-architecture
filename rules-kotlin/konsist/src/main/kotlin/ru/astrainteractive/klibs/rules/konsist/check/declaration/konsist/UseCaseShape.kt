package ru.astrainteractive.klibs.rules.konsist.check.declaration.konsist

import com.lemonappdev.konsist.api.KoModifier
import com.lemonappdev.konsist.api.declaration.KoClassDeclaration
import com.lemonappdev.konsist.api.declaration.KoFunctionDeclaration
import ru.astrainteractive.klibs.rules.konsist.check.konsist.ArchitectureCheck
import ru.astrainteractive.klibs.rules.konsist.check.konsist.UnitSources
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import ru.astrainteractive.klibs.rules.konsist.check.model.util.simpleName
import ru.astrainteractive.klibs.rules.konsist.check.model.util.topLevelClasses
import ru.astrainteractive.klibs.rules.konsist.layout.model.Kind
import ru.astrainteractive.klibs.rules.konsist.layout.model.Placement
import ru.astrainteractive.klibs.rules.konsist.layout.model.UnitKind

/**
 * Reports a use case out of shape: `<Verb><Object>UseCase` outside a `usecase` package, a class in `usecase`
 * not named so, more than one public entry or no `operator fun invoke`, and a use case that takes another use
 * case, which an entry point calls instead.
 */
internal class UseCaseShape : ArchitectureCheck {
    override val unitKinds: Set<UnitKind> = UnitKind.entries.toSet()

    private fun findEntryMessage(useCase: KoClassDeclaration): String? {
        val entries = useCase.functions(includeNested = false, includeLocal = false)
            .filterNot(KoFunctionDeclaration::hasPrivateModifier)
        val isOperatorInvoke = entries.singleOrNull()
            ?.let { entry -> entry.name == INVOKE && entry.hasModifier(KoModifier.OPERATOR) }
            ?: false
        if (isOperatorInvoke) return null
        val names = entries.joinToString(transform = KoFunctionDeclaration::name).ifEmpty { "none" }
        return "${useCase.name} has the public functions $names: a use case has one public entry, " +
            "operator fun invoke."
    }

    private fun findMessages(declaration: KoClassDeclaration, placement: Placement?): List<String> {
        val isNamedUseCase = declaration.name.endsWith(USE_CASE_SUFFIX)
        val isInUseCaseKind = placement?.kind == Kind.USECASE
        if (!isNamedUseCase) {
            if (!isInUseCaseKind) return emptyList()
            return listOf("Name ${declaration.name} <Verb><Object>UseCase, or move it out of the usecase package.")
        }
        val placementMessage = "Move ${declaration.name} to the usecase package of its concept."
            .takeUnless { isInUseCaseKind }
        val calledMessages = declaration.primaryConstructor?.parameters.orEmpty()
            .map { parameter -> parameter.type.simpleName }
            .filter { typeName -> typeName.endsWith(USE_CASE_SUFFIX) }
            .map { typeName ->
                "${declaration.name} takes $typeName: a use case never calls another use case; " +
                    "the entry point calls both."
            }
        return listOfNotNull(placementMessage, findEntryMessage(declaration)) + calledMessages
    }

    override fun check(sources: UnitSources): List<Violation> {
        return sources.mainFiles.flatMap { file ->
            val placement = sources.placementOf(file)
            file.topLevelClasses().flatMap { declaration ->
                findMessages(declaration, placement).map { message ->
                    Violation(
                        RULE,
                        sources.relativePath(file),
                        declaration.name,
                        message
                    )
                }
            }
        }
    }

    private companion object {
        const val RULE = "UseCaseShape"
        const val USE_CASE_SUFFIX = "UseCase"
        const val INVOKE = "invoke"
    }
}
