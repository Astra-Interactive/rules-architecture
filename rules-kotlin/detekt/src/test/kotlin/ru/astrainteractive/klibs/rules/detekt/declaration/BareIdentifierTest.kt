@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.detekt.declaration

import dev.detekt.api.Config
import dev.detekt.api.Finding
import dev.detekt.test.lint
import org.junit.jupiter.api.Test
import kotlin.test.assertEquals

internal class BareIdentifierTest {
    private val rule = BareIdentifier(Config.empty)

    @Test
    fun GIVEN_bare_identifiers_WHEN_linted_THEN_reports_each() {
        val code = """
            package com.example.jail.model

            data class JailInmate(val id: String, val jailId: Long?, val ownerId: java.util.UUID) {
                val cellId: Int = 0
            }

            interface HasId {
                val id: String
            }

            class PlayerId(val uuid: String)

            data class JailId(val value: Long)
        """.trimIndent()

        val messages = rule.lint(code).map(Finding::message)

        assertEquals(7, messages.size)
        assertEquals(
            "'id' is an identifier typed String: give it a typed identifier, a value class such as PlayerId, " +
                "instead of the bare primitive.",
            messages.first()
        )
        assertEquals("'JailId' is an identifier: make it a @JvmInline value class over the primitive.", messages.last())
    }

    @Test
    fun GIVEN_typed_identifiers_and_boundaries_WHEN_linted_THEN_reports_nothing() {
        val modelCode = """
            package com.example.jail.model

            @JvmInline
            value class PlayerId(val uuid: String)

            inline class LegacyId(val value: Int)

            enum class KindId { A }

            data class JailInmate(val id: PlayerId, val name: String, override val ownerId: Long, valid: Int) : Owned {
                fun find(playerId: String) {
                    val localId = playerId
                }
            }

            @Serializable
            data class JailDto(val id: String)

            val topId: String = ""

            class Lookup(val id: () -> String)
        """.trimIndent()
        val tableCode = """
            package com.example.jail.data.exposed.table

            class JailRow(val id: Long)
        """.trimIndent()

        assertEquals(0, rule.lint(modelCode).size + rule.lint(tableCode).size)
    }
}
