package ru.astrainteractive.klibs.rules.structure.nesting.structure

import ru.astrainteractive.klibs.rules.structure.unit.api.StructureCheck
import ru.astrainteractive.klibs.rules.structure.unit.model.BuildUnit
import ru.astrainteractive.klibs.rules.structure.unit.model.StructureViolation

/**
 * Reports a unit named after a Multiplatform target (`android`, `ios`, `jvm`, …): an app's platform code is a source
 * set of the unit it belongs to, never a unit. Each platform's application is an entry point, which is not judged.
 */
public class PlatformSourceSetUnit : StructureCheck {
    override fun check(unit: BuildUnit): List<StructureViolation> {
        if (unit.isRoot || unit.isEntryPoint || unit.name !in TARGETS) return emptyList()
        val message = "Move the code of ${unit.path} to the ${unit.name}Main source set of the unit it belongs to: " +
            "platform code is a source set, never a unit."
        return listOf(
            StructureViolation(
                RULE,
                ".",
                unit.path,
                message
            )
        )
    }

    private companion object {
        const val RULE = "PlatformSourceSetUnit"

        @JvmField
        val TARGETS = setOf(
            "android",
            "apple",
            "desktop",
            "ios",
            "js",
            "jvm",
            "native",
            "wasm"
        )
    }
}
