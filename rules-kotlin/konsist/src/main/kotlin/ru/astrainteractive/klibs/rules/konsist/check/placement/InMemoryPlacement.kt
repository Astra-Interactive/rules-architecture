package ru.astrainteractive.klibs.rules.konsist.check.placement

import ru.astrainteractive.klibs.rules.konsist.check.ArchitectureCheck
import ru.astrainteractive.klibs.rules.konsist.check.model.UnitSources
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import ru.astrainteractive.klibs.rules.konsist.check.model.packageName
import ru.astrainteractive.klibs.rules.konsist.check.model.topLevelClasses
import ru.astrainteractive.klibs.rules.konsist.layout.UnitKind

/**
 * Reports an in-memory implementation (`InMemory*`) outside `data/memory` of the main sources: it is production
 * code that tests and demos share, not a test double.
 */
internal class InMemoryPlacement : ArchitectureCheck {
    override val unitKinds: Set<UnitKind> = UnitKind.entries.toSet()

    override fun check(sources: UnitSources): List<Violation> {
        val inMain = sources.mainFiles
            .filterNot { file -> file.packageName.endsWith(MEMORY_PACKAGE) }
            .flatMap { file ->
                file.topLevelClasses()
                    .filter { declaration -> IN_MEMORY_NAME.matches(declaration.name) }
                    .map { declaration ->
                        val message = "Move ${declaration.name} to the data/memory package of its context."
                        Violation(RULE, sources.relativePath(file), declaration.name, message)
                    }
            }
        val inTests = sources.testFiles.flatMap { file ->
            file.topLevelClasses()
                .filter { declaration -> IN_MEMORY_NAME.matches(declaration.name) }
                .map { declaration ->
                    val message = "Move ${declaration.name} to data/memory of the main sources: an in-memory " +
                        "implementation is production code."
                    Violation(RULE, sources.relativePath(file), declaration.name, message)
                }
        }
        return inMain + inTests
    }

    private companion object {
        const val RULE = "InMemoryPlacement"
        const val MEMORY_PACKAGE = "data.memory"

        @JvmField
        val IN_MEMORY_NAME = Regex("InMemory[A-Z]\\w*")
    }
}
