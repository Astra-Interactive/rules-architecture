@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.declaration.detekt

import dev.detekt.api.Config
import dev.detekt.api.Finding
import dev.detekt.test.lint
import org.junit.jupiter.api.Test
import kotlin.test.assertEquals

internal class SerialNameNotProvidedTest {
    private val rule = SerialNameNotProvided(Config.empty)

    @Test
    fun GIVEN_serializable_class_with_constructor_property_without_serial_name_WHEN_linted_THEN_reports_one_finding_naming_the_property() {
        val code = """
            import kotlinx.serialization.Serializable

            @Serializable
            data class PlayerDto(val uuid: String)
        """.trimIndent()

        assertEquals(listOf("Annotate 'uuid' with @SerialName."), rule.lint(code).map(Finding::message))
    }

    @Test
    fun GIVEN_serializable_class_with_every_property_annotated_WHEN_linted_THEN_reports_nothing() {
        val code = """
            import kotlinx.serialization.SerialName
            import kotlinx.serialization.Serializable

            @Serializable
            data class PlayerDto(
                @SerialName("uuid") val uuid: String,
                @SerialName("name") val name: String
            )
        """.trimIndent()

        assertEquals(0, rule.lint(code).size)
    }

    @Test
    fun GIVEN_transient_property_without_serial_name_WHEN_linted_THEN_reports_nothing() {
        val code = """
            import kotlinx.serialization.Serializable
            import kotlinx.serialization.Transient

            @Serializable
            data class PlayerDto(
                @Transient val cachedName: String = ""
            )
        """.trimIndent()

        assertEquals(0, rule.lint(code).size)
    }

    @Test
    fun GIVEN_class_without_serializable_WHEN_linted_THEN_reports_nothing() {
        val code = """
            class Player(val uuid: String)
        """.trimIndent()

        assertEquals(0, rule.lint(code).size)
    }

    @Test
    fun GIVEN_body_property_without_serial_name_WHEN_linted_THEN_reports_one_finding() {
        val code = """
            import kotlinx.serialization.Serializable

            @Serializable
            class PlayerDto {
                val uuid: String = ""
            }
        """.trimIndent()

        assertEquals(1, rule.lint(code).size)
    }

    @Test
    fun GIVEN_serializable_enum_with_entries_WHEN_linted_THEN_reports_nothing() {
        val code = """
            import kotlinx.serialization.Serializable

            @Serializable
            enum class Role { USER, ADMIN }
        """.trimIndent()

        assertEquals(0, rule.lint(code).size)
    }

    @Test
    fun GIVEN_constructor_parameter_that_is_neither_val_nor_var_WHEN_linted_THEN_reports_nothing() {
        val code = """
            import kotlinx.serialization.Serializable

            @Serializable
            class PlayerDto(name: String)
        """.trimIndent()

        assertEquals(0, rule.lint(code).size)
    }

    @Test
    fun GIVEN_mix_of_annotated_unannotated_and_transient_properties_WHEN_linted_THEN_reports_exactly_one_finding() {
        val code = """
            import kotlinx.serialization.SerialName
            import kotlinx.serialization.Serializable
            import kotlinx.serialization.Transient

            @Serializable
            data class PlayerDto(
                @SerialName("uuid") val uuid: String,
                val name: String,
                @Transient val cachedName: String = ""
            )
        """.trimIndent()

        assertEquals(listOf("Annotate 'name' with @SerialName."), rule.lint(code).map(Finding::message))
    }
}
