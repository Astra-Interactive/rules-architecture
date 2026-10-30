package ru.astrainteractive.klibs.rules.structure.unit.structure

private val PLATFORM_UNIT_NAMES = setOf(
    "bukkit",
    "bungee",
    "fabric",
    "forge",
    "neoforge",
    "paper",
    "spigot",
    "velocity"
)

/**
 * Whether a unit whose path ends in [name] is the platform unit of a sub-context split by platform (`gui/bukkit`).
 */
public fun isPlatformUnitName(name: String): Boolean = name in PLATFORM_UNIT_NAMES
