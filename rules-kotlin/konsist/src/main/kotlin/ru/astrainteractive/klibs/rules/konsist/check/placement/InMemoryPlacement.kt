package ru.astrainteractive.klibs.rules.konsist.check.placement

import com.lemonappdev.konsist.api.declaration.KoFileDeclaration
import ru.astrainteractive.klibs.rules.konsist.check.ArchitectureCheck
import ru.astrainteractive.klibs.rules.konsist.check.model.UnitSources
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import ru.astrainteractive.klibs.rules.konsist.check.model.packageName
import ru.astrainteractive.klibs.rules.konsist.check.model.topLevelClasses
import ru.astrainteractive.klibs.rules.konsist.layout.UnitKind

/**
 * Reports an in-memory implementation (`InMemory*`) outside a `memory` package of the main sources: it is
 * production code that tests and demos share, not a test double.
 */
internal class InMemoryPlacement : ArchitectureCheck {
    override val unitKinds: Set<UnitKind> = UnitKind.entries.toSet()

    private fun inMemoryNames(file: KoFileDeclaration): List<String> {
        return file.topLevelClasses()
            .map { declaration -> declaration.name }
            .filter { name -> IN_MEMORY_NAME.matches(name) }
    }

    override fun check(sources: UnitSources): List<Violation> {
        val inMain = sources.mainFiles
            .filterNot { file -> file.packageName.substringAfterLast('.') == MEMORY_SEGMENT }
            .flatMap { file ->
                inMemoryNames(file).map { name ->
                    val message = "Move $name to the memory package of its concept."
                    Violation(RULE, sources.relativePath(file), name, message)
                }
            }
        val inTests = sources.testFiles.flatMap { file ->
            inMemoryNames(file).map { name ->
                val message = "Move $name to a memory package of the main sources: an in-memory implementation " +
                    "is production code."
                Violation(RULE, sources.relativePath(file), name, message)
            }
        }
        return inMain + inTests
    }

    private companion object {
        const val RULE = "InMemoryPlacement"
        const val MEMORY_SEGMENT = "memory"

        @JvmField
        val IN_MEMORY_NAME = Regex("""InMemory[A-Z]\w*""")
    }
}
