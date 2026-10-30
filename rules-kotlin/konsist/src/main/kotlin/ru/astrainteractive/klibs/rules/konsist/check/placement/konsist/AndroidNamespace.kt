package ru.astrainteractive.klibs.rules.konsist.check.placement.konsist

import ru.astrainteractive.klibs.rules.konsist.check.konsist.ArchitectureCheck
import ru.astrainteractive.klibs.rules.konsist.check.konsist.UnitSources
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import ru.astrainteractive.klibs.rules.konsist.layout.model.UnitKind

/**
 * Reports an Android unit whose `namespace` differs from its root package: an Android unit's `namespace` is its
 * root package.
 */
internal class AndroidNamespace : ArchitectureCheck {
    override val unitKinds: Set<UnitKind> = UnitKind.entries.toSet()

    override fun check(sources: UnitSources): List<Violation> {
        val namespace = sources.androidNamespace ?: return emptyList()
        val rootPackage = sources.layout.rootPackage
        if (namespace == rootPackage) return emptyList()
        val message = "Set the namespace of this Android unit to its root package $rootPackage instead of " +
            "$namespace."
        val violation = sources.createExternalViolation(
            RULE,
            BUILD_FILE,
            namespace,
            message
        )
        return listOf(violation)
    }

    private companion object {
        const val RULE = "AndroidNamespace"
        const val BUILD_FILE = "build.gradle.kts"
    }
}
