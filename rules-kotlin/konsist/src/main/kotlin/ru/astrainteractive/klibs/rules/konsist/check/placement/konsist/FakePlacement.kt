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
 * Reports a test fake (`Fake*`) outside a `fake` package of the test tree, which sits inside the concept of the
 * contract it replaces, and a fake in the main sources, which belongs to the test tree or to `testFixtures`. A
 * `<feature>/fake` unit, whose root package ends with `fake`, holds the shared fakes of a Multiplatform unit in
 * its main sources.
 */
internal class FakePlacement : ArchitectureCheck {
    override val unitKinds: Set<UnitKind> = UnitKind.entries.toSet()

    private fun isArchitectureCheck(declaration: KoClassDeclaration): Boolean {
        return declaration.parents().any { parent -> parent.name == ARCHITECTURE_CHECK }
    }

    private fun findFakes(
        sources: UnitSources,
        files: List<KoFileDeclaration>,
        message: (String) -> String
    ): List<Violation> {
        return files.flatMap { file ->
            val declarations = file.topLevelClasses().filterNot(::isArchitectureCheck) +
                file.objects(includeNested = false)
            declarations
                .filterNot { declaration -> declaration.name.endsWith(TEST_SUFFIX) }
                .filter { declaration -> FAKE_NAME.matches(declaration.name) }
                .map { declaration ->
                    sources.createViolation(
                        RULE,
                        file,
                        declaration,
                        declaration.name,
                        message.invoke(declaration.name)
                    )
                }
        }
    }

    private fun findTestFakes(sources: UnitSources): List<Violation> {
        val files = sources.testFiles.filterNot { file -> file.packageName.substringAfterLast('.') == FAKE_SEGMENT }
        return findFakes(sources, files) { name ->
            "Move $name to a fake package inside the concept of the contract it replaces."
        }
    }

    private fun findMainFakes(sources: UnitSources): List<Violation> {
        if (sources.layout.rootPackage.substringAfterLast('.') == FAKE_SEGMENT) return emptyList()
        return findFakes(sources, sources.mainFiles) { name ->
            "Move $name out of the main sources: a fake lives in the test tree, in a fake package inside the " +
                "concept of the contract it replaces, or in testFixtures of the unit that owns the contract."
        }
    }

    override fun check(sources: UnitSources): List<Violation> {
        return findTestFakes(sources) + findMainFakes(sources)
    }

    private companion object {
        const val RULE = "FakePlacement"
        const val FAKE_SEGMENT = "fake"
        const val ARCHITECTURE_CHECK = "ArchitectureCheck"
        const val TEST_SUFFIX = "Test"

        @JvmField
        val FAKE_NAME = Regex("""Fake[A-Z]\w*""")
    }
}
