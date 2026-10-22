@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.konsist.check.placement

import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
import ru.astrainteractive.klibs.rules.konsist.check.model.SourceTree
import ru.astrainteractive.klibs.rules.konsist.layout.UnitKind
import java.io.File
import kotlin.test.assertEquals

internal class TestOwnerBehaviourTest {

    @Test
    fun GIVEN_tests_of_owners_without_behaviour_WHEN_checked_THEN_reports_each(@TempDir directory: File) {
        val sources = SourceTree(directory)
            .main("x.jail.model/Jail.kt", "data class Jail(val name: String)")
            .main("x.jail.model/Stage.kt", "enum class Stage { OFFER }")
            .main(
                "x.jail.model/Plan.kt",
                """
                data class Plan(val name: String) {
                    val isFree = true
                }
                """
            )
            .main(
                "x.jail.model/Kind.kt",
                """
                enum class Kind {
                    A;

                    fun next(): Kind = A
                }
                """
            )
            .main("x.jail.usecase/JailUseCase.kt", "class JailUseCase")
            .test("x.jail.model/JailTest.kt", "class JailTest")
            .test("x.jail.model/StageTest.kt", "class StageTest")
            .test("x.jail.model/PlanTest.kt", "class PlanTest")
            .test("x.jail.model/KindTest.kt", "class KindTest")
            .test("x.jail.usecase/JailUseCaseTest.kt", "class JailUseCaseTest")
            .test("x.jail.model/Jail.kt", "class Jail")
            .load(UnitKind.CONTEXT, rootPackage = "x")

        assertEquals(
            listOf("JailTest", "StageTest"),
            TestOwnerBehaviour().check(sources).map { violation -> violation.subject }.sorted()
        )
    }
}
