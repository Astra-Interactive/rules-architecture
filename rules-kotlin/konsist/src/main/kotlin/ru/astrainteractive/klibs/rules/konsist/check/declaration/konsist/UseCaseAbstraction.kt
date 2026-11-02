package ru.astrainteractive.klibs.rules.konsist.check.declaration.konsist

import com.lemonappdev.konsist.api.KoModifier
import ru.astrainteractive.klibs.rules.core.unit.model.ProductKind
import ru.astrainteractive.klibs.rules.core.unit.model.UnitKind
import ru.astrainteractive.klibs.rules.konsist.check.konsist.ArchitectureCheck
import ru.astrainteractive.klibs.rules.konsist.check.konsist.UnitSources
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import ru.astrainteractive.klibs.rules.konsist.check.model.util.topLevelClasses

/**
 * Reports an interface or an abstract class named `…UseCase`: a use case gets no interface. The one exception is
 * an app's use case that another unit calls, whose interface sits in `usecase` of the owner's `api` unit and its
 * `Default…` implementation in `impl`.
 */
internal class UseCaseAbstraction : ArchitectureCheck {
    override val unitKinds: Set<UnitKind> = UnitKind.entries.toSet()

    override fun check(sources: UnitSources): List<Violation> {
        if (sources.context.publishedLibrary) return emptyList()
        return sources.mainFiles.flatMap { file ->
            val useCaseInterfaces = file.interfaces(includeNested = false)
                .filter { declaration -> declaration.name.endsWith(USE_CASE_SUFFIX) }
            if (sources.kind == UnitKind.API && useCaseInterfaces.isNotEmpty()) {
                requireNotNull(sources.context.productKind) {
                    "Set linter.productKind(ProductKind.APP or ProductKind.PLATFORM_PLUGIN) " +
                        "before checking use-case interfaces in an API unit"
                }
            }
            val isApiUseCase = sources.kind == UnitKind.API && sources.context.productKind == ProductKind.APP
            val interfaces = file.interfaces(includeNested = false).takeUnless { isApiUseCase }.orEmpty()
            val abstractClasses = file.topLevelClasses()
                .filter { declaration -> declaration.hasModifier(KoModifier.ABSTRACT) }
            interfaces.plus(abstractClasses)
                .filter { declaration -> declaration.name.endsWith(USE_CASE_SUFFIX) }
                .map { declaration ->
                    val message = "Make ${declaration.name} a final class with operator fun invoke: a use case gets " +
                        "no interface; a test runs it with fakes of the contracts it takes."
                    sources.createViolation(
                        RULE,
                        file,
                        declaration,
                        declaration.name,
                        message
                    )
                }
        }
    }

    private companion object {
        const val RULE = "UseCaseAbstraction"
        const val USE_CASE_SUFFIX = "UseCase"
    }
}
