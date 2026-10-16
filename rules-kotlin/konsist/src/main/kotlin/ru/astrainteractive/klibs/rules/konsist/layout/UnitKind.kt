package ru.astrainteractive.klibs.rules.konsist.layout

/**
 * The kind of build unit, which decides what may sit at its root and which checks apply.
 */
internal enum class UnitKind(val cliName: String) {
    CONTEXT("context"),
    LIBRARY("library"),
    API("api"),
    CONTRACT("contract"),
    ENTRY_POINT("entry-point");

    companion object {
        fun fromCliName(cliName: String): UnitKind? = entries.find { kind -> kind.cliName == cliName }
    }
}
