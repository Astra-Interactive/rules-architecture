@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.core.technology.util

import kotlin.test.Test
import kotlin.test.assertEquals

class TechnologyNamesTest {

    @Test
    fun GIVEN_package_words_WHEN_judged_THEN_only_library_file_and_memory_names_are_technologies() {
        val words = listOf(
            "exposed",
            "memory",
            "file",
            "table",
            "network",
            "rating"
        )

        assertEquals(
            listOf(
                "exposed",
                "memory",
                "file"
            ),
            words.filter(::isTechnologyName)
        )
    }
}
