package ru.astrainteractive.klibs.rules.konsist.check.placement.konsist

import com.lemonappdev.konsist.api.declaration.KoClassDeclaration
import com.lemonappdev.konsist.api.declaration.KoFileDeclaration
import ru.astrainteractive.klibs.rules.konsist.check.konsist.ArchitectureCheck
import ru.astrainteractive.klibs.rules.konsist.check.konsist.UnitSources
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import ru.astrainteractive.klibs.rules.konsist.check.model.util.packageName
import ru.astrainteractive.klibs.rules.konsist.check.model.util.topLevelClasses
import ru.astrainteractive.klibs.rules.konsist.layout.model.UnitKind

/**
 * Reports an in-memory implementation (`InMemory*`) outside a `memory` package of the main sources: it is
 * production code that tests and demos share, not a test double.
 */
internal class InMemoryPlacement : ArchitectureCheck {
    override val unitKinds: Set<UnitKind> = UnitKind.entries.toSet()

    private fun inMemoryNames(file: KoFileDeclaration): List<String> {
        return file.topLevelClasses()
            .filterNot { declaration -> declaration.parents().any { parent -> parent.name == ARCHITECTURE_CHECK } }
            .map(KoClassDeclaration::name)
            .filterNot { name -> name.endsWith(TEST_SUFFIX) }
            .filter(IN_MEMORY_NAME::matches)
    }

    override fun check(sources: UnitSources): List<Violation> {
        val inMain = sources.mainFiles
            .filterNot { file -> file.packageName.substringAfterLast('.') == MEMORY_SEGMENT }
            .flatMap { file ->
                inMemoryNames(file).map { name ->
                    val message = "Move $name to the memory package of its concept."
                    Violation(
                        RULE,
                        sources.relativePath(file),
                        name,
                        message
                    )
                }
            }
        val inTests = sources.testFiles.flatMap { file ->
            inMemoryNames(file).map { name ->
                val message = "Move $name to a memory package of the main sources: an in-memory implementation " +
                    "is production code."
                Violation(
                    RULE,
                    sources.relativePath(file),
                    name,
                    message
                )
            }
        }
        return inMain + inTests
    }

    private companion object {
        const val RULE = "InMemoryPlacement"
        const val MEMORY_SEGMENT = "memory"
        const val ARCHITECTURE_CHECK = "ArchitectureCheck"
        const val TEST_SUFFIX = "Test"

        @JvmField
        val IN_MEMORY_NAME = Regex("""InMemory[A-Z]\w*""")
    }
}
