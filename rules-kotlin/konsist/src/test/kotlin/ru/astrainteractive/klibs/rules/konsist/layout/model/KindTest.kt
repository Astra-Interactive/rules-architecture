@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.konsist.layout.model

import org.junit.jupiter.api.Test
import kotlin.test.assertEquals

internal class KindTest {

    @Test
    fun GIVEN_project_words_WHEN_segments_looked_up_THEN_they_join_the_default_kinds() {
        val vocabulary = Vocabulary(
            entryPointWords = setOf("slot"),
            technologyWords = setOf("mongo"),
            pluralConcepts = emptySet(),
            platformClasses = emptySet(),
            extensionKinds = emptySet(),
            roleKinds = emptySet()
        )

        assertEquals(
            listOf(
                Kind.ENTRY_POINT,
                Kind.IMPLEMENTATION,
                Kind.ENTRY_POINT,
                Kind.MODEL,
                null,
                null,
                null
            ),
            listOf(
                "slot",
                "mongo",
                "command",
                "model",
                "rating",
                "jail",
                "konsist"
            ).map { segment -> Kind.fromSegment(segment, vocabulary) }
        )
    }

    @Test
    fun GIVEN_configured_role_kind_words_WHEN_segments_looked_up_THEN_only_those_words_are_role_kinds() {
        val vocabulary = Vocabulary(
            entryPointWords = emptySet(),
            technologyWords = emptySet(),
            pluralConcepts = emptySet(),
            platformClasses = emptySet(),
            extensionKinds = emptySet(),
            roleKinds = setOf("limiter", "codec")
        )

        assertEquals(
            listOf(
                Kind.ROLE,
                Kind.ROLE,
                null,
                Kind.UTIL
            ),
            listOf(
                "limiter",
                "codec",
                "registry",
                "util"
            ).map { segment -> Kind.fromSegment(segment, vocabulary) }
        )
    }
}
