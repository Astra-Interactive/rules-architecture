package ru.astrainteractive.klibs.rules.konsist.check.placement.konsist

import com.lemonappdev.konsist.api.KoModifier
import com.lemonappdev.konsist.api.declaration.KoClassDeclaration
import com.lemonappdev.konsist.api.declaration.KoFileDeclaration
import com.lemonappdev.konsist.api.provider.KoNameProvider
import ru.astrainteractive.klibs.rules.konsist.check.konsist.ArchitectureCheck
import ru.astrainteractive.klibs.rules.konsist.check.konsist.UnitSources
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import ru.astrainteractive.klibs.rules.konsist.check.model.util.baseName
import ru.astrainteractive.klibs.rules.konsist.check.model.util.topLevelClasses
import ru.astrainteractive.klibs.rules.konsist.layout.model.UnitKind

/**
 * Reports a test file `<Owner>Test.kt` whose owner has no behaviour of its own: a data class without a body or an
 * enum without functions. Its rules live in `util`, and their tests are named after that file.
 */
internal class TestOwnerBehaviour : ArchitectureCheck {
    override val unitKinds: Set<UnitKind> = UnitKind.entries.toSet()

    private fun hasNoBehaviour(declaration: KoClassDeclaration): Boolean {
        if (!declaration.hasModifier(KoModifier.DATA) && !declaration.hasModifier(KoModifier.ENUM)) return false
        return declaration.functions(includeNested = true, includeLocal = false).isEmpty()
    }

    override fun check(sources: UnitSources): List<Violation> {
        val dataOwners = sources.mainFiles
            .flatMap(KoFileDeclaration::topLevelClasses)
            .filter(::hasNoBehaviour)
            .map(KoNameProvider::name)
            .toSet()
        return sources.testFiles
            .filter { file -> file.baseName.removeSuffix(TEST_SUFFIX) in dataOwners }
            .filter { file -> file.baseName.endsWith(TEST_SUFFIX) }
            .map { file ->
                val owner = file.baseName.removeSuffix(TEST_SUFFIX)
                val message = "Delete or rename ${file.baseName}: $owner has no behaviour of its own; test its rules " +
                    "in the test of their util file."
                Violation(
                    RULE,
                    sources.relativePath(file),
                    file.baseName,
                    message
                )
            }
    }

    private companion object {
        const val RULE = "TestOwnerBehaviour"
        const val TEST_SUFFIX = "Test"
    }
}
