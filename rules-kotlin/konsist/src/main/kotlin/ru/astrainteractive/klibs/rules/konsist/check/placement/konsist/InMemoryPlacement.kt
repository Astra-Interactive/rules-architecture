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

    private fun inMemoryDeclarations(file: KoFileDeclaration): List<KoClassDeclaration> {
        return file.topLevelClasses()
            .filterNot { declaration -> declaration.parents().any { parent -> parent.name == ARCHITECTURE_CHECK } }
            .filterNot { declaration -> declaration.name.endsWith(TEST_SUFFIX) }
            .filter { declaration -> IN_MEMORY_NAME.matches(declaration.name) }
    }

    override fun check(sources: UnitSources): List<Violation> {
        val inMain = sources.mainFiles
            .filterNot { file -> file.packageName.substringAfterLast('.') == MEMORY_SEGMENT }
            .flatMap { file ->
                inMemoryDeclarations(file).map { declaration ->
                    val message = "Move ${declaration.name} to the memory package of its concept."
                    sources.createViolation(
                        RULE,
                        file,
                        declaration,
                        declaration.name,
                        message
                    )
                }
            }
        val inTests = sources.testFiles.flatMap { file ->
            inMemoryDeclarations(file).map { declaration ->
                val message = "Move ${declaration.name} to a memory package of the main sources: an in-memory " +
                    "implementation is production code."
                sources.createViolation(
                    RULE,
                    file,
                    declaration,
                    declaration.name,
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
