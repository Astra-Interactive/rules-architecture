package ru.astrainteractive.klibs.rules.gradle.linter.model

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
