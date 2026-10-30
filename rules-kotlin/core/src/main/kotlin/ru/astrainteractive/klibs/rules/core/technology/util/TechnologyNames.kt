package ru.astrainteractive.klibs.rules.core.technology.util

/**
 * The technology names of the package-layout rule: the libraries a technology package or unit is named after, and
 * `file` and `memory`.
 */
public val TECHNOLOGY_NAMES: Set<String> = setOf(
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
    "sqldelight",
)

public fun isTechnologyName(word: String): Boolean = word in TECHNOLOGY_NAMES
