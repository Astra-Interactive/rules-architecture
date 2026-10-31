package ru.astrainteractive.klibs.rules.konsist.check.placement.konsist

import com.lemonappdev.konsist.api.declaration.KoFileDeclaration
import com.lemonappdev.konsist.api.provider.KoNameProvider
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
 * its main sources. A fake in the test sources of a contract another unit declares belongs to the `testFixtures` of
 * that unit, shared with every unit that tests against it.
 */
@Suppress("FakePlacement")
internal class FakePlacement : ArchitectureCheck {
    override val unitKinds: Set<UnitKind> = UnitKind.entries.toSet()

    private fun findFakes(
        sources: UnitSources,
        files: List<KoFileDeclaration>,
        message: (String) -> String
    ): List<Violation> {
        return files.flatMap { file ->
            val declarations = file.topLevelClasses() +
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
        return findFakes(
            sources,
            files
        ) { name ->
            "Move $name to a fake package inside the concept of the contract it replaces."
        }
    }

    private fun findMainFakes(sources: UnitSources): List<Violation> {
        if (sources.layout.rootPackage.substringAfterLast('.') == FAKE_SEGMENT) return emptyList()
        return findFakes(
            sources,
            sources.mainFiles
        ) { name ->
            "Move $name out of the main sources: a fake lives in the test tree, in a fake package inside the " +
                "concept of the contract it replaces, or in testFixtures of the unit that owns the contract."
        }
    }

    private fun interfaceNames(files: List<KoFileDeclaration>): Set<String> {
        return files.flatMap { file -> file.interfaces(includeNested = false) }.map(KoNameProvider::name).toSet()
    }

    private fun findForeignFakes(sources: UnitSources): List<Violation> {
        val foreignContracts = interfaceNames(sources.dependencyFiles) - interfaceNames(sources.mainFiles)
        return sources.testFiles
            .filterNot { file -> sources.testSourceSetOf(file) == TEST_FIXTURES }
            .flatMap { file ->
                file.topLevelClasses()
                    .filter { declaration -> FAKE_NAME.matches(declaration.name) }
                    .mapNotNull { declaration ->
                        val contract = declaration.parents()
                            .map { parent -> parent.name.substringBefore('<') }
                            .firstOrNull { name -> name in foreignContracts }
                            ?: return@mapNotNull null
                        val message = "Move ${declaration.name} to testFixtures of the unit that owns $contract: a " +
                            "fake that tests of other units need is shared from there, never copied into each one."
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

    private fun findProductCode(sources: UnitSources): List<Violation> {
        if (sources.layout.rootPackage.substringAfterLast('.') != FAKE_SEGMENT) return emptyList()
        return sources.mainFiles.flatMap { file ->
            file.topLevelClasses()
                .plus(file.objects(includeNested = false))
                .filterNot { declaration -> FAKE_NAME.matches(declaration.name) }
                .map { declaration ->
                    val message = "Move ${declaration.name} out of this fake unit: a <feature>/fake unit is tooling " +
                        "and holds the shared fakes only, no product code."
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
        return findTestFakes(sources) + findMainFakes(sources) + findForeignFakes(sources) + findProductCode(sources)
    }

    private companion object {
        const val RULE = "FakePlacement"
        const val FAKE_SEGMENT = "fake"
        const val TEST_SUFFIX = "Test"
        const val TEST_FIXTURES = "testFixtures"

        @JvmField
        val FAKE_NAME = Regex("""Fake[A-Z]\w*""")
    }
}
