package ru.astrainteractive.klibs.rules.konsist.check.declaration

import com.lemonappdev.konsist.api.KoModifier
import com.lemonappdev.konsist.api.provider.KoNameProvider
import ru.astrainteractive.klibs.rules.konsist.check.ArchitectureCheck
import ru.astrainteractive.klibs.rules.konsist.check.model.UnitSources
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import ru.astrainteractive.klibs.rules.konsist.check.model.topLevelClasses
import ru.astrainteractive.klibs.rules.konsist.layout.UnitKind

/**
 * Reports an interface or an abstract class named `…UseCase`: a use case gets no interface.
 */
internal class UseCaseAbstraction : ArchitectureCheck {
    override val unitKinds: Set<UnitKind> = UnitKind.entries.toSet()

    override fun check(sources: UnitSources): List<Violation> {
        return sources.mainFiles.flatMap { file ->
            val interfaces = file.interfaces(includeNested = false).map(KoNameProvider::name)
            val abstractClasses = file.topLevelClasses()
                .filter { declaration -> declaration.hasModifier(KoModifier.ABSTRACT) }
                .map(KoNameProvider::name)
            interfaces.plus(abstractClasses)
                .filter { name -> name.endsWith(USE_CASE_SUFFIX) }
                .map { name ->
                    val message = "Make $name a final class with operator fun invoke: a use case gets no interface; " +
                        "a test runs it with fakes of the contracts it takes."
                    Violation(
                        RULE,
                        sources.relativePath(file),
                        name,
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
