package ru.astrainteractive.klibs.rules.konsist.layout.model

/**
 * The project's own words, added to the defaults of [Kind]: entry-point kinds (`slot`, `router`), technology packages
 * (`mongo`), inherently plural concepts (`goods`) and the supertypes of the classes the platform constructs
 * (`WatchFaceService`).
 */
internal data class Vocabulary(
    val entryPointWords: Set<String>,
    val technologyWords: Set<String>,
    val pluralConcepts: Set<String>,
    val platformClasses: Set<String>
)
