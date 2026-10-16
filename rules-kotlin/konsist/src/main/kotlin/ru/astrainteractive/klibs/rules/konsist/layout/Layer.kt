package ru.astrainteractive.klibs.rules.konsist.layout

/**
 * The rows of the dependency-direction table of the cross-context rule.
 */
internal enum class Layer {
    MODEL,
    CONTRACT,
    USE_CASE,
    TECHNOLOGY,
    MAPPING,
    ENTRY_POINT,
    UTIL,
    DI
}
