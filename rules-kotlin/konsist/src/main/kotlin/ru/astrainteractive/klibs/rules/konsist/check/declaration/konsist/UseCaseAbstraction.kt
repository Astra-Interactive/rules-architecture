package ru.astrainteractive.klibs.rules.konsist.check.declaration.konsist

import com.lemonappdev.konsist.api.KoModifier
import ru.astrainteractive.klibs.rules.konsist.check.konsist.ArchitectureCheck
import ru.astrainteractive.klibs.rules.konsist.check.konsist.UnitSources
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import ru.astrainteractive.klibs.rules.konsist.check.model.util.topLevelClasses
import ru.astrainteractive.klibs.rules.konsist.layout.model.UnitKind

/**
 * Reports an interface or an abstract class named `…UseCase`: a use case gets no interface.
 */
internal class UseCaseAbstraction : ArchitectureCheck {
    override val unitKinds: Set<UnitKind> = UnitKind.entries.toSet()

    override fun check(sources: UnitSources): List<Violation> {
        return sources.mainFiles.flatMap { file ->
            val interfaces = file.interfaces(includeNested = false)
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
