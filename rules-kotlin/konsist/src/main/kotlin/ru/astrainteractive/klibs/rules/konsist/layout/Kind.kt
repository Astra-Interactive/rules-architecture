package ru.astrainteractive.klibs.rules.konsist.layout

/**
 * A kind package, grouped by the row of the direction table of `arch-context.md` it falls under.
 *
 * Entry-point and technology words are open vocabularies; the sets hold the words the rules and the projects
 * use. A segment in no set is not a known kind.
 */
internal enum class Kind(val words: Set<String>) {
    MODEL(setOf("model")),
    CONTRACT(setOf("api")),
    USECASE(setOf("usecase")),
    UTIL(setOf("util")),
    DI(setOf("di")),

    /**
     * Compose only: the values several composables of a level share, and in the design-system unit the theme.
     */
    THEME(setOf("theme")),
    IMPLEMENTATION(
        setOf(
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
            "stub",
            "table",
        )
    ),
    ENTRY_POINT(
        setOf(
            "command",
            "component",
            "composable",
            "context",
            "error",
            "event",
            "expansion",
            "gui",
            "job",
            "menu",
            "migration",
            "route",
            "router",
            "screen",
            "service",
            "slot",
            "viewmodel",
            "warmup",
        )
    );

    companion object {
        fun fromSegment(segment: String): Kind? = entries.find { kind -> segment in kind.words }
    }
}
