package ru.astrainteractive.klibs.rules.konsist.check.placement.konsist

import com.lemonappdev.konsist.api.declaration.KoFileDeclaration
import ru.astrainteractive.klibs.rules.core.unit.model.UnitKind
import ru.astrainteractive.klibs.rules.konsist.check.konsist.ArchitectureCheck
import ru.astrainteractive.klibs.rules.konsist.check.konsist.UnitSources
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import ru.astrainteractive.klibs.rules.konsist.check.model.util.baseName
import ru.astrainteractive.klibs.rules.konsist.check.model.util.packageName
import ru.astrainteractive.klibs.rules.konsist.check.model.util.topLevelNames

/**
 * Reports a test file `<Owner>Test.kt` in another package or source set than its owner. A test whose owner is not
 * found in the main sources is not judged; unscoped source directories keep the package-only behavior.
 */
internal class TestMirror : ArchitectureCheck {
    override val unitKinds: Set<UnitKind> = UnitKind.entries.toSet()

    private class OwnerLocation(val name: String, val packageName: String, val sourceSet: String?)

    private fun ownersByName(sources: UnitSources): Map<String, Set<OwnerLocation>> {
        return sources.mainFiles
            .flatMap { file ->
                val names = file.topLevelNames() + file.baseName
                names.map { name ->
                    OwnerLocation(
                        name,
                        file.packageName,
                        sources.mainSourceSetOf(file)
                    )
                }
            }
            .groupBy(OwnerLocation::name)
            .mapValues { entry -> entry.value.toSet() }
    }

    private fun expectedTestSourceSets(sources: UnitSources, mainSourceSet: String): Set<String> {
        if (mainSourceSet == "main") return setOf("test")
        if (mainSourceSet == "commonMain") return setOf("commonTest")
        val compilations = sources.context.compilations
        val production = compilations.filter { compilation ->
            !compilation.test && mainSourceSet in compilation.sourceSets
        }
        return production.flatMap { mainCompilation ->
            val target = mainCompilation.name.substringBeforeLast(':', missingDelimiterValue = "")
            if (target.isEmpty()) return@flatMap emptyList()
            compilations.filter { testCompilation ->
                if (!testCompilation.test) return@filter false
                val testTarget = testCompilation.name.substringBeforeLast(':', missingDelimiterValue = "")
                if (testTarget != target) return@filter false
                mainSourceSet in testCompilation.sourceSets
            }.flatMap { testCompilation ->
                (testCompilation.sourceSets.keys - mainCompilation.sourceSets.keys - "commonTest").toList()
            }
        }.toSet()
    }

    private fun violation(sources: UnitSources, file: KoFileDeclaration, message: String): Violation {
        return sources.createViolation(
            RULE,
            file,
            file.baseName,
            message
        )
    }

    override fun check(sources: UnitSources): List<Violation> {
        val ownersByName = ownersByName(sources)
        return sources.testFiles
            .filter { file -> file.baseName.endsWith(TEST_SUFFIX) }
            .mapNotNull { file ->
                val ownerName = file.baseName.removeSuffix(TEST_SUFFIX)
                val owners = ownersByName[ownerName] ?: return@mapNotNull null
                val packages = owners.map(OwnerLocation::packageName).toSet()
                if (file.packageName !in packages) {
                    val message = "Move ${file.baseName} to ${packages.sorted().first()}, the package of $ownerName."
                    return@mapNotNull violation(
                        sources,
                        file,
                        message
                    )
                }
                val testSourceSet = sources.testSourceSetOf(file) ?: return@mapNotNull null
                val matchingOwners = owners.filter { owner -> owner.packageName == file.packageName }
                val expected = matchingOwners.mapNotNull(OwnerLocation::sourceSet)
                    .flatMap { mainSourceSet -> expectedTestSourceSets(sources, mainSourceSet) }
                    .toSet()
                if (expected.isEmpty() || testSourceSet in expected) return@mapNotNull null
                val message = "Move ${file.baseName} to ${expected.sorted().first()}, the test source set of " +
                    "$ownerName in ${file.packageName}."
                violation(
                    sources,
                    file,
                    message
                )
            }
    }

    private companion object {
        const val RULE = "TestMirror"
        const val TEST_SUFFIX = "Test"
    }
}
