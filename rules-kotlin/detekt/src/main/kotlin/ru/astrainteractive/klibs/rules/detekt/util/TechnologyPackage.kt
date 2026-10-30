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
 *
 * @param projectWords the project's own technology words, from the `technologyWords` setting of the rule
 */
internal fun isTechnologySegment(segment: String, projectWords: List<String>): Boolean {
    return segment in TECHNOLOGY_SEGMENTS || segment in projectWords
}
