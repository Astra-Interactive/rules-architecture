@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.structure.graph

import org.junit.jupiter.api.Test
import ru.astrainteractive.klibs.rules.structure.unit.BuildUnit
import java.io.File
import kotlin.test.assertEquals

internal class EntryPointDependencyTest {

    @Test
    fun GIVEN_dependency_on_an_entry_point_WHEN_checked_THEN_reports_it() {
        val unit = BuildUnit(
            path = ":modules:command",
            directory = File("/project"),
            isEntryPoint = false,
            isRoot = false,
            projectDependencies = setOf(
                ":instances:bukkit",
                ":modules:gui:api",
                ":instancesx"
            )
        )

        assertEquals(
            listOf(
                "Drop the dependency on the entry point :instances:bukkit: nothing depends on the entry point; " +
                    "depend on the api unit that holds what you need."
            ),
            EntryPointDependency().check(unit).map { violation -> violation.message }
        )
    }
}
