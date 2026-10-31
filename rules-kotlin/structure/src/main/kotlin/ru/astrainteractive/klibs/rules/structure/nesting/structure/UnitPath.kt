package ru.astrainteractive.klibs.rules.structure.nesting.structure

import ru.astrainteractive.klibs.rules.structure.unit.api.StructureCheck
import ru.astrainteractive.klibs.rules.structure.unit.model.BuildUnit
import ru.astrainteractive.klibs.rules.structure.unit.model.StructureViolation

/**
 * Reports a sub-context unit at the root of the build: units live in a container, `modules/<sub-context>` or
 * `modules/<context>/<sub-context>`. The entry point, a published library and the root project are not judged.
 */
public class UnitPath : StructureCheck {
    override fun check(unit: BuildUnit): List<StructureViolation> {
        if (unit.isRoot || unit.isEntryPoint || unit.isLibrary) return emptyList()
        val segments = unit.path.split(':').filter(String::isNotEmpty)
        if (segments.size > 1) return emptyList()
        val message = "Move ${unit.path} into a container, modules/${unit.name}: a sub-context unit lives under " +
            "modules/<sub-context> or modules/<context>/<sub-context>."
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
        const val RULE = "UnitPath"
    }
}
