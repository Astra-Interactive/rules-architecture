package ru.astrainteractive.klibs.rules.konsist.check.placement

import com.lemonappdev.konsist.api.declaration.KoFileDeclaration
import ru.astrainteractive.klibs.rules.konsist.check.ArchitectureCheck
import ru.astrainteractive.klibs.rules.konsist.check.model.UnitSources
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import ru.astrainteractive.klibs.rules.konsist.check.model.packageName
import ru.astrainteractive.klibs.rules.konsist.check.model.topLevelClassAndObjectNames
import ru.astrainteractive.klibs.rules.konsist.layout.UnitKind

/**
 * Reports a test fake (`Fake*`) outside a `fake` package of the test tree, which sits inside the concept of the
 * contract it replaces, and a fake in the main sources, which belongs to the test tree or to `testFixtures`. A
 * `<feature>/fake` unit, whose root package ends with `fake`, holds the shared fakes of a Multiplatform unit in
 * its main sources.
 */
internal class FakePlacement : ArchitectureCheck {
    override val unitKinds: Set<UnitKind> = UnitKind.entries.toSet()

    private fun findFakes(
        sources: UnitSources,
        files: List<KoFileDeclaration>,
        message: (String) -> String
    ): List<Violation> {
        return files.flatMap { file ->
            file.topLevelClassAndObjectNames()
                .filter(FAKE_NAME::matches)
                .map { name ->
                    Violation(
                        RULE,
                        sources.relativePath(file),
                        name,
                        message.invoke(name)
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

        @JvmField
        val FAKE_NAME = Regex("""Fake[A-Z]\w*""")
    }
}
