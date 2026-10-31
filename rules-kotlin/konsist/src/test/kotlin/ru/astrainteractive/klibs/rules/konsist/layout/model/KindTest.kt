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
            extensionKinds = emptySet()
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
}
