package ru.astrainteractive.klibs.rules.detekt.util

private val TECHNOLOGY_SEGMENTS = setOf(
    "cache4k",
    "datastore",
    "entity",
    "exposed",
    "file",
    "krate",
    "ktor",
    "memory",
    "okhttp",
    "retrofit",
    "room",
    "sqldelight",
    "table",
)

/**
 * Whether [segment] is a package named after a technology, the boundary where a library's own types are allowed.
 */
internal fun isTechnologySegment(segment: String): Boolean = segment in TECHNOLOGY_SEGMENTS
