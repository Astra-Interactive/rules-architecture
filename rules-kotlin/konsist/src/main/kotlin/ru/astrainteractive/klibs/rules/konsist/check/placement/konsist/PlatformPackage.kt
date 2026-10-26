package ru.astrainteractive.klibs.rules.konsist.check.placement.konsist

import com.lemonappdev.konsist.api.declaration.KoFileDeclaration
import ru.astrainteractive.klibs.rules.konsist.check.konsist.ArchitectureCheck
import ru.astrainteractive.klibs.rules.konsist.check.konsist.UnitSources
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import ru.astrainteractive.klibs.rules.konsist.check.model.util.packageName
import ru.astrainteractive.klibs.rules.konsist.layout.model.UnitKind

/**
 * Reports a package named after a platform (`android`, `ios`, `jvm`, …) in a Multiplatform unit, one with more than
 * one main source directory: platform code goes to its source set at the same package path.
 */
internal class PlatformPackage : ArchitectureCheck {
    override val unitKinds: Set<UnitKind> = UnitKind.entries.toSet()

    override fun check(sources: UnitSources): List<Violation> {
        if (sources.mainDirectories.size < MULTIPLATFORM_SOURCE_SETS) return emptyList()
        return sources.mainFiles
            .distinctBy(KoFileDeclaration::packageName)
            .mapNotNull { file ->
                val platform = file.packageName.split('.').firstOrNull { segment -> segment in PLATFORMS }
                    ?: return@mapNotNull null
                val message = "Move the code of the package '$platform' to the ${platform}Main source set at the " +
                    "same package path: platform code is a source set, never a package."
                sources.createViolation(
                    RULE,
                    file,
                    file.packageName,
                    message
                )
            }
    }

    private companion object {
        const val RULE = "PlatformPackage"
        const val MULTIPLATFORM_SOURCE_SETS = 2

        @JvmField
        val PLATFORMS = setOf(
            "android",
            "ios",
            "jvm",
            "js",
            "wasm",
            "desktop",
            "native",
            "apple"
        )
    }
}
