package ru.astrainteractive.klibs.rules.structure.unit.structure

import ru.astrainteractive.klibs.rules.core.technology.util.isTechnologyName

/**
 * Whether a unit whose path ends in [name] is wholly one technology's implementation (`data/exposed`).
 */
public fun isTechnologyUnitName(name: String): Boolean = isTechnologyName(name)
