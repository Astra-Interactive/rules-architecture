package ru.astrainteractive.klibs.rules.konsist.check.placement.konsist

import com.lemonappdev.konsist.api.declaration.KoClassDeclaration
import com.lemonappdev.konsist.api.declaration.KoFileDeclaration
import ru.astrainteractive.klibs.rules.core.unit.model.UnitKind
import ru.astrainteractive.klibs.rules.konsist.check.konsist.ArchitectureCheck
import ru.astrainteractive.klibs.rules.konsist.check.konsist.UnitSources
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import ru.astrainteractive.klibs.rules.konsist.check.model.util.packageName
import ru.astrainteractive.klibs.rules.konsist.check.model.util.topLevelClasses

/**
 * Reports an in-memory implementation (`InMemory*`) outside a `memory` package, in the main sources and in a test
 * source set alike: it sits in the `memory` package of its concept in the source set of the code that constructs it.
 */
@Suppress("InMemoryPlacement")
internal class InMemoryPlacement : ArchitectureCheck {
    override val unitKinds: Set<UnitKind> = UnitKind.entries.toSet()

    private fun inMemoryDeclarations(file: KoFileDeclaration): List<KoClassDeclaration> {
        return file.topLevelClasses()
            .filterNot { declaration -> declaration.name.endsWith(TEST_SUFFIX) }
            .filter { declaration -> IN_MEMORY_NAME.matches(declaration.name) }
    }

    private fun checkFiles(
        sources: UnitSources,
        files: List<KoFileDeclaration>,
        findDestination: (KoFileDeclaration) -> String
    ): List<Violation> {
        return files
            .filterNot { file -> file.packageName.substringAfterLast('.') == MEMORY_SEGMENT }
            .flatMap { file ->
                inMemoryDeclarations(file).map { declaration ->
                    val message = "Move ${declaration.name} to the memory package of its concept" +
                        "${findDestination.invoke(file)}."
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

    override fun check(sources: UnitSources): List<Violation> {
        val inMain = checkFiles(
            sources,
            sources.mainFiles
        ) { "" }
        val inTests = checkFiles(
            sources,
            sources.testFiles
        ) { file ->
            val sourceSet = sources.testSourceSetOf(file) ?: UNNAMED_SOURCE_SET
            " in the $sourceSet source set"
        }
        return inMain + inTests
    }

    private companion object {
        const val RULE = "InMemoryPlacement"
        const val MEMORY_SEGMENT = "memory"
        const val TEST_SUFFIX = "Test"
        const val UNNAMED_SOURCE_SET = "same"

        @JvmField
        val IN_MEMORY_NAME = Regex("""InMemory[A-Z]\w*""")
    }
}
