package ru.astrainteractive.klibs.rules.structure.unit.structure

private val TECHNOLOGY_UNIT_NAMES = setOf(
    "cache4k",
    "datastore",
    "exposed",
    "file",
    "krate",
    "ktor",
    "memory",
    "okhttp",
    "retrofit",
    "room",
    "sqldelight"
)

/**
 * Whether a unit whose path ends in [name] is wholly one technology's implementation (`data/exposed`).
 */
public fun isTechnologyUnitName(name: String): Boolean = name in TECHNOLOGY_UNIT_NAMES
