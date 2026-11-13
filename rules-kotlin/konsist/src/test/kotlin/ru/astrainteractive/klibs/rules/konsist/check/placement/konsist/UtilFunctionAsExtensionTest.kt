@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.konsist.check.placement.konsist

import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
import ru.astrainteractive.klibs.rules.core.unit.model.UnitKind
import ru.astrainteractive.klibs.rules.konsist.check.model.SourceTree
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import java.io.File
import kotlin.test.assertEquals

internal class UtilFunctionAsExtensionTest {

    private fun writeJailModels(directory: File): SourceTree {
        return SourceTree(directory)
            .main("com.example.jail.model/JailInmate.kt", "data class JailInmate(val days: Int)")
            .main("com.example.jail.model/Money.kt", "data class Money(val cents: Long)")
            .main("com.example.jail.model/Claim.kt", "data class Claim(val chunks: List<ClaimChunk>)")
            .main("com.example.jail.model/ClaimChunk.kt", "data class ClaimChunk(val x: Int)")
            .main("com.example.jail.model/Dictionary.kt", "data class Dictionary(val owner: UserId)")
            .main("com.example.jail.model/UserId.kt", "@JvmInline\nvalue class UserId(val value: String)")
            .main("com.example.jail.model/JailId.kt", "@JvmInline\nvalue class JailId(val value: String)")
    }

    private fun checkUtil(directory: File, code: String): List<Violation> {
        val sources = writeJailModels(directory)
            .main("com.example.jail.util/JailRule.kt", code)
            .load(UnitKind.CONTEXT, rootPackage = "com.example")
        return UtilFunctionAsExtension().check(sources)
    }

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
                    "one type extends that type.",
                "Write isWithinLimit as an extension of JailInmate: fun JailInmate.isWithinLimit(), a rule about " +
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

    @Test
    fun GIVEN_top_level_functions_returning_a_model_parameter_type_WHEN_checked_THEN_reports_each_as_its_extension(
        @TempDir directory: File
    ) {
        val violations = checkUtil(
            directory,
            """
            fun addChunk(claim: Claim, chunk: ClaimChunk): Claim = claim

            fun releaseInmate(inmate: JailInmate, deposit: Money): JailInmate? = inmate

            fun mergeClaims(first: Claim, second: Claim): Result<Claim> = Result.success(first)

            fun selectNextJail(current: JailId, other: JailId): JailId = current
            """
        )

        assertEquals(
            listOf(
                "Write addChunk as an extension of Claim: fun Claim.addChunk(), a transition is an extension of " +
                    "the type it returns.",
                "Write releaseInmate as an extension of JailInmate: fun JailInmate.releaseInmate(), a transition " +
                    "is an extension of the type it returns.",
                "Write mergeClaims as an extension of Claim: fun Claim.mergeClaims(), a transition is an extension " +
                    "of the type it returns."
            ),
            violations.map(Violation::message)
        )
    }

    @Test
    fun GIVEN_top_level_functions_with_one_model_parameter_among_others_WHEN_checked_THEN_reports_each(
        @TempDir directory: File
    ) {
        val violations = checkUtil(
            directory,
            """
            fun canRead(dictionary: Dictionary, reader: UserId?): Boolean = true

            fun formatSentence(inmate: JailInmate, locale: java.util.Locale, days: List<Int>): String = ""

            fun isSameOwner(owner: UserId, reader: UserId): Boolean = true
            """
        )

        assertEquals(
            listOf(
                "Write canRead as an extension of Dictionary: fun Dictionary.canRead(), a rule about one type " +
                    "extends that type.",
                "Write formatSentence as an extension of JailInmate: fun JailInmate.formatSentence(), a rule " +
                    "about one type extends that type."
            ),
            violations.map(Violation::message)
        )
    }

    @Test
    fun GIVEN_extensions_reading_a_second_model_type_WHEN_checked_THEN_reports_each_as_a_top_level_function(
        @TempDir directory: File
    ) {
        val violations = checkUtil(
            directory,
            """
            fun JailInmate.isBailCovered(deposit: Money): Boolean = true

            fun Claim?.countChunks(chunk: ClaimChunk?, limit: Int): Int = 0
            """
        )

        assertEquals(
            listOf("JailInmate.isBailCovered", "Claim.countChunks"),
            violations.map(Violation::subject)
        )
        assertEquals(
            "Write isBailCovered as a top-level function of JailInmate and Money in a file named after the " +
                "decision: a rule that reads two model types is a top-level function.",
            violations.first().message
        )
    }

    @Test
    fun GIVEN_create_and_parse_functions_WHEN_checked_THEN_exempts_only_those_followed_by_a_type_name(
        @TempDir directory: File
    ) {
        val violations = checkUtil(
            directory,
            """
            fun createClaim(chunk: ClaimChunk): Result<Claim> = Result.success(Claim(listOf(chunk)))

            fun parseMoney(template: Money, raw: String): Result<Money> = Result.success(template)

            fun createdBy(inmate: JailInmate): UserId = UserId("")

            fun parse(inmate: JailInmate): String = ""
            """
        )

        assertEquals(listOf("createdBy", "parse"), violations.map(Violation::subject))
    }

    @Test
    fun GIVEN_top_level_functions_over_two_model_types_returning_neither_WHEN_checked_THEN_reports_nothing(
        @TempDir directory: File
    ) {
        val violations = checkUtil(
            directory,
            """
            fun isBailCovered(inmate: JailInmate, deposit: Money): Boolean = true

            fun countChunks(claim: Claim, chunk: ClaimChunk?): Int = 0

            fun splitClaim(claim: Claim, chunk: ClaimChunk): List<Claim> = listOf(claim)
            """
        )

        assertEquals(emptyList(), violations)
    }

    @Test
    fun GIVEN_extensions_returning_their_receiver_type_with_other_model_parameters_WHEN_checked_THEN_reports_nothing(
        @TempDir directory: File
    ) {
        val violations = checkUtil(
            directory,
            """
            fun Claim.addChunk(chunk: ClaimChunk): Claim = this

            fun JailInmate.payBail(deposit: Money): Result<JailInmate> = Result.success(this)

            fun Money.plus(other: Money): Money? = this
            """
        )

        assertEquals(emptyList(), violations)
    }

    @Test
    fun GIVEN_extensions_over_identifiers_primitives_library_types_or_foreign_receivers_WHEN_checked_THEN_accepts_them(
        @TempDir directory: File
    ) {
        val violations = checkUtil(
            directory,
            """
            fun Dictionary.canRead(reader: UserId?): Boolean = true

            fun JailInmate.isWithinLimit(limit: Int, now: java.time.Instant, jail: JailId): Boolean = true

            fun String.isBailCovered(inmate: JailInmate, deposit: Money): Boolean = true

            fun List<JailInmate>.countCovered(deposit: Money): Int = 0
            """
        )

        assertEquals(emptyList(), violations)
    }

    @Test
    fun GIVEN_extensions_whose_model_parameters_are_all_of_the_receiver_type_WHEN_checked_THEN_reports_nothing(
        @TempDir directory: File
    ) {
        val violations = checkUtil(
            directory,
            """
            fun Money.exceeds(limit: Money): Boolean = true

            fun Claim?.overlaps(other: Claim?, days: Int): Boolean = true

            fun Money.isBetween(low: Money, high: Money?, jail: JailId): Boolean = true
            """
        )

        assertEquals(emptyList(), violations)
    }

    @Test
    fun GIVEN_extension_with_a_receiver_type_parameter_and_another_model_parameter_WHEN_checked_THEN_reports_it(
        @TempDir directory: File
    ) {
        val violations = checkUtil(
            directory,
            """
            fun Money.isCoveredFor(limit: Money, inmate: JailInmate): Boolean = true
            """
        )

        assertEquals(
            listOf(
                "Write isCoveredFor as a top-level function of Money and JailInmate in a file named after the " +
                    "decision: a rule that reads two model types is a top-level function."
            ),
            violations.map(Violation::message)
        )
    }

    @Test
    fun GIVEN_top_level_functions_whose_model_parameters_are_all_of_one_type_WHEN_checked_THEN_reports_each(
        @TempDir directory: File
    ) {
        val violations = checkUtil(
            directory,
            """
            fun exceeds(amount: Money, limit: Money): Boolean = true

            fun overlaps(first: Claim?, second: Claim, days: Int): Boolean = true
            """
        )

        assertEquals(
            listOf(
                "Write exceeds as an extension of Money: fun Money.exceeds(), a rule about one type extends that " +
                    "type.",
                "Write overlaps as an extension of Claim: fun Claim.overlaps(), a rule about one type extends that " +
                    "type."
            ),
            violations.map(Violation::message)
        )
    }

    @Test
    fun GIVEN_extensions_returning_another_model_parameter_type_WHEN_checked_THEN_reports_each_as_its_extension(
        @TempDir directory: File
    ) {
        val violations = checkUtil(
            directory,
            """
            fun JailInmate.cover(deposit: Money): Money = deposit

            fun Claim?.split(chunk: ClaimChunk, limit: Int): Result<ClaimChunk> = Result.success(chunk)
            """
        )

        assertEquals(
            listOf("JailInmate.cover", "Claim.split"),
            violations.map(Violation::subject)
        )
        assertEquals(
            listOf(
                "Write cover as an extension of Money: fun Money.cover(), a transition is an extension of the type " +
                    "it returns.",
                "Write split as an extension of ClaimChunk: fun ClaimChunk.split(), a transition is an extension of " +
                    "the type it returns."
            ),
            violations.map(Violation::message)
        )
    }

    @Test
    fun GIVEN_top_level_function_with_two_values_of_one_model_and_one_of_another_WHEN_checked_THEN_reports_nothing(
        @TempDir directory: File
    ) {
        val violations = checkUtil(
            directory,
            """
            fun isCovered(amount: Money, limit: Money, inmate: JailInmate): Boolean = true
            """
        )

        assertEquals(emptyList(), violations)
    }
}
