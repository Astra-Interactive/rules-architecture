package ru.astrainteractive.klibs.rules.konsist.layout.model

import ru.astrainteractive.klibs.rules.core.technology.util.TECHNOLOGY_NAMES

/**
 * A kind package, grouped by the row of the direction table of `arch-context.md` it falls under.
 *
 * Entry-point and technology words are open vocabularies; the sets hold the words the rules and the projects
 * use. Role kinds have no default words. A segment in no set is not a known kind.
 */
internal enum class Kind(val words: Set<String>) {
    MODEL(setOf("model")),
    CONTRACT(setOf("api")),
    USECASE(setOf("usecase")),
    UTIL(setOf("util")),
    DI(setOf("di")),
    FAKE(setOf("fake")),

    /**
     * Compose only: the values several composables of a level share, and in the design-system unit the theme.
     */
    THEME(setOf("theme")),
    IMPLEMENTATION(
        TECHNOLOGY_NAMES + setOf(
            "entity",
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
            "screen",
            "service",
            "slot",
            "viewmodel",
            "warmup",
        )
    ),

    /**
     * A class no other kind fits, in a kind named by the last word of its name; the words are the project's only.
     */
    ROLE(emptySet());

    companion object {
        /**
         * Compose's `composable`, the one kind that may hold sub-packages.
         */
        const val COMPOSABLE_WORD = "composable"

        /**
         * The words `arch-package.md` (Concepts and kinds 10) forbids as the name of a package inside a unit.
         */
        val FORBIDDEN_WORDS: Set<String> = setOf(
            "adapter",
            "aggregate",
            "application",
            "common",
            "controller",
            "dao",
            "domain",
            "dto",
            "entrypoint",
            "enum",
            "exception",
            "factory",
            "helpers",
            "impl",
            "in",
            "inbound",
            "infrastructure",
            "internal",
            "misc",
            "out",
            "outbound",
            "port",
            "presentation",
            "repository",
            "shared",
            "spi",
            "type",
            "ui",
            "validation",
            "valueobject",
        )

        /**
         * The kind a package segment names, from the project's own words in [vocabulary] first, then from the
         * defaults.
         */
        fun fromSegment(segment: String, vocabulary: Vocabulary): Kind? {
            return when (segment) {
                in vocabulary.entryPointWords -> ENTRY_POINT
                in vocabulary.technologyWords -> IMPLEMENTATION
                in vocabulary.roleKinds -> ROLE
                else -> entries.find { kind -> segment in kind.words }
            }
        }
    }
}
