package ru.astrainteractive.klibs.rules.konsist.check.model

import ru.astrainteractive.klibs.rules.core.unit.model.UnitKind

/**
 * A unit of a worked tree of the package-layout skill.
 *
 * @property path the unit's directory as the tree writes it (`modules/gui/api`)
 */
internal class SkillUnit(
    val path: String,
    val kind: UnitKind,
    val rootPackage: String,
    val files: List<SkillFile>
) {
    val gradlePath: String = ":" + path.replace('/', ':')
}
