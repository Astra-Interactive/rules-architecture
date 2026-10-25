package ru.astrainteractive.klibs.rules.konsist.layout.model

/**
 * The kind of build unit, which decides the exceptions to the concept → kind shape.
 */
internal enum class UnitKind(val cliName: String) {
    CONTEXT("context"),
    LIBRARY("library"),
    API("api"),
    CONTRACT("contract"),
    ENTRY_POINT("entry-point"),

    /**
     * A unit that is wholly one technology's implementation (`data/exposed`): its concept packages hold the
     * implementations directly.
     */
    TECHNOLOGY("technology");

    companion object {
        fun fromCliName(cliName: String): UnitKind? = entries.find { kind -> kind.cliName == cliName }
    }
}
