@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.konsist.layout.util

import org.junit.jupiter.api.Test
import ru.astrainteractive.klibs.rules.konsist.layout.model.Kind
import ru.astrainteractive.klibs.rules.konsist.layout.model.Vocabulary
import kotlin.test.assertEquals

internal class VocabularyExtTest {

    @Test
    fun GIVEN_project_words_WHEN_segments_looked_up_THEN_they_join_the_default_kinds() {
        val vocabulary = Vocabulary(
            entryPointWords = setOf("slot"),
            technologyWords = setOf("mongo"),
            pluralConcepts = emptySet()
        )

        assertEquals(
            listOf(
                Kind.ENTRY_POINT,
                Kind.IMPLEMENTATION,
                Kind.ENTRY_POINT,
                Kind.MODEL,
                null,
                null
            ),
            listOf(
                "slot",
                "mongo",
                "command",
                "model",
                "rating",
                "jail"
            ).map(vocabulary::kindOf)
        )
    }
}
