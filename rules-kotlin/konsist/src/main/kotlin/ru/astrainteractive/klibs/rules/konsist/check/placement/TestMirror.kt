package ru.astrainteractive.klibs.rules.konsist.check.placement

import ru.astrainteractive.klibs.rules.konsist.check.ArchitectureCheck
import ru.astrainteractive.klibs.rules.konsist.check.model.UnitSources
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import ru.astrainteractive.klibs.rules.konsist.check.model.baseName
import ru.astrainteractive.klibs.rules.konsist.check.model.packageName
import ru.astrainteractive.klibs.rules.konsist.check.model.topLevelNames
import ru.astrainteractive.klibs.rules.konsist.layout.UnitKind

/**
 * Reports a test file `<Owner>Test.kt` in another package than its owner: the test tree mirrors the main tree
 * package for package. A test whose owner is not found in the main sources is not judged.
 */
internal class TestMirror : ArchitectureCheck {
    override val unitKinds: Set<UnitKind> = UnitKind.entries.toSet()

    private class OwnerLocation(val name: String, val packageName: String)

    private fun packagesByName(sources: UnitSources): Map<String, Set<String>> {
        return sources.mainFiles
            .flatMap { file ->
                val names = file.topLevelNames() + file.baseName
                names.map { name -> OwnerLocation(name, file.packageName) }
            }
            .groupBy(keySelector = OwnerLocation::name, valueTransform = OwnerLocation::packageName)
            .mapValues { entry -> entry.value.toSet() }
    }

    override fun check(sources: UnitSources): List<Violation> {
        val packagesByName = packagesByName(sources)
        return sources.testFiles
            .filter { file -> file.baseName.endsWith(TEST_SUFFIX) }
            .mapNotNull { file ->
                val ownerName = file.baseName.removeSuffix(TEST_SUFFIX)
                val ownerPackages = packagesByName[ownerName] ?: return@mapNotNull null
                if (file.packageName in ownerPackages) return@mapNotNull null
                val message = "Move ${file.baseName} to ${ownerPackages.sorted().first()}, the package of $ownerName."
                Violation(
                    RULE,
                    sources.relativePath(file),
                    file.baseName,
                    message
                )
            }
    }

    private companion object {
        const val RULE = "TestMirror"
        const val TEST_SUFFIX = "Test"
    }
}
