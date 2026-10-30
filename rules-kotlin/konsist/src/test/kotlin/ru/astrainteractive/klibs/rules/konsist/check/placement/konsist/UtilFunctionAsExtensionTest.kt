@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.konsist.check.placement.konsist

import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
import ru.astrainteractive.klibs.rules.konsist.check.model.SourceTree
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import ru.astrainteractive.klibs.rules.konsist.layout.model.UnitKind
import java.io.File
import kotlin.test.assertEquals

internal class UtilFunctionAsExtensionTest {

    @Test
    fun GIVEN_util_functions_taking_one_model_WHEN_checked_THEN_reports_them(@TempDir directory: File) {
        val sources = SourceTree(directory)
            .main("com.example.jail.model/JailInmate.kt", "data class JailInmate(val days: Int)")
            .main(
                "com.example.jail.util/Sentence.kt",
                """
                fun isSentenceOver(inmate: JailInmate?): Boolean = true

                fun isWithinLimit(inmate: JailInmate, limit: Int): Boolean = true

                fun JailInmate.isFree(): Boolean = true

                fun parse(raw: String): Int = 0

                fun now(): Int = 0
                """
            )
            .main("com.example.jail.command/JailCommand.kt", "fun describe(inmate: JailInmate): String = \"\"")
            .load(UnitKind.CONTEXT, rootPackage = "com.example")

        assertEquals(
            listOf(
                "Write isSentenceOver as an extension of JailInmate: fun JailInmate.isSentenceOver(), a rule about " +
                    "one type extends that type."
            ),
            UtilFunctionAsExtension().check(sources).map(Violation::message)
        )
    }

    @Test
    fun GIVEN_util_function_taking_a_dependency_model_WHEN_checked_THEN_reports_it(@TempDir directory: File) {
        val sources = SourceTree(directory)
            .dependency("x.core.settings.model/AstraRatingConfig.kt", "data class AstraRatingConfig(val max: Int)")
            .main(
                "x.gui.api.rating.util/RatingMessage.kt",
                "fun isMessageAllowed(config: AstraRatingConfig): Boolean = config.max > 0"
            )
            .load(UnitKind.API, rootPackage = "x.gui.api")

        assertEquals(
            listOf("isMessageAllowed"),
            UtilFunctionAsExtension().check(sources).map(Violation::subject)
        )
    }
}
