package ru.astrainteractive.klibs.rules.konsist.check.placement

import ru.astrainteractive.klibs.rules.konsist.check.ArchitectureCheck
import ru.astrainteractive.klibs.rules.konsist.check.model.UnitSources
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import ru.astrainteractive.klibs.rules.konsist.check.model.packageName
import ru.astrainteractive.klibs.rules.konsist.check.model.topLevelClasses
import ru.astrainteractive.klibs.rules.konsist.layout.UnitKind

/**
 * Reports a test fake (`Fake*`) outside a `fake` package, which sits inside the concept of the contract it
 * replaces.
 */
internal class FakePlacement : ArchitectureCheck {
    override val unitKinds: Set<UnitKind> = UnitKind.entries.toSet()

    override fun check(sources: UnitSources): List<Violation> = sources.testFiles
        .filterNot { file -> file.packageName.substringAfterLast('.') == FAKE_SEGMENT }
        .flatMap { file ->
            val names = file.topLevelClasses().map { declaration -> declaration.name } +
                file.objects(includeNested = false).map { declaration -> declaration.name }
            names
                .filter { name -> FAKE_NAME.matches(name) }
                .map { name ->
                    val message = "Move $name to a fake package inside the concept of the contract it replaces."
                    Violation(RULE, sources.relativePath(file), name, message)
                }
        }

    private companion object {
        const val RULE = "FakePlacement"
        const val FAKE_SEGMENT = "fake"

        @JvmField
        val FAKE_NAME = Regex("Fake[A-Z]\\w*")
    }
}
