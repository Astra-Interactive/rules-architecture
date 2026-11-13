package ru.astrainteractive.klibs.rules.konsist.layout.model

/**
 * The project's own words, added to the defaults of [Kind]: entry-point kinds (`slot`, `router`), technology packages
 * (`mongo`), inherently plural concepts (`goods`), the supertypes of the classes the platform constructs
 * (`WatchFaceService`), the UI kinds whose parts are extension functions (`slot`: `fun SlotContext.backSlot()`) and
 * the role kinds, each named by the last word of its classes (`limiter`: `VoteCooldownLimiter`).
 */
internal data class Vocabulary(
    val entryPointWords: Set<String>,
    val technologyWords: Set<String>,
    val pluralConcepts: Set<String>,
    val platformClasses: Set<String>,
    val extensionKinds: Set<String>,
    val roleKinds: Set<String>
)
