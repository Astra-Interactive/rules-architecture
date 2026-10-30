package ru.astrainteractive.klibs.rules.detekt.util

import ru.astrainteractive.klibs.rules.core.technology.util.isTechnologyName

private val STORAGE_KINDS = setOf("entity", "table")

/**
 * Whether [segment] is a package named after a technology, the boundary where a library's own types are allowed.
 *
 * @param projectWords the project's own technology words, from the `technologyWords` setting of the rule
 */
internal fun isTechnologySegment(segment: String, projectWords: List<String>): Boolean {
    return isTechnologyName(segment) || segment in STORAGE_KINDS || segment in projectWords
}
