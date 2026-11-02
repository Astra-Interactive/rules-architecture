@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.core.unit.util

import org.junit.jupiter.api.Test
import ru.astrainteractive.klibs.rules.core.unit.model.ProductKind
import ru.astrainteractive.klibs.rules.core.unit.model.UnitContext
import ru.astrainteractive.klibs.rules.core.unit.model.UnitKind
import java.io.ByteArrayOutputStream
import java.io.DataOutputStream
import java.util.Base64
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
import kotlin.test.assertNull

internal class StringExtTest {
    private fun policyWire(disabledCount: Int, vararg fields: String): String {
        val bytes = ByteArrayOutputStream()
        DataOutputStream(bytes).use { output ->
            output.writeInt(UNIT_CONTEXT_TRANSPORT_VERSION)
            output.writeUTF("impl-ui")
            output.writeUTF("APP")
            output.writeBoolean(false)
            output.writeInt(0)
            output.writeInt(0)
            output.writeUTF("LOCAL")
            output.writeUTF("NONE")
            output.writeInt(disabledCount)
            fields.toList().chunked(2).forEach { entry ->
                output.writeUTF(entry[0])
                output.writeUTF(entry[1])
            }
            output.writeInt(0)
        }
        return Base64.getEncoder().encodeToString(bytes.toByteArray())
    }

    private fun replaceWireText(encoded: String, old: String, replacement: String): String {
        val bytes = Base64.getDecoder().decode(encoded)
        val oldBytes = old.encodeToByteArray()
        val start = bytes.indices.first { offset ->
            bytes.drop(offset).take(oldBytes.size) == oldBytes.toList()
        }
        replacement.encodeToByteArray().copyInto(bytes, start)
        return Base64.getEncoder().encodeToString(bytes)
    }

    @Test
    fun GIVEN_command_line_names_WHEN_parsed_THEN_maps_known_names_and_rejects_unknown_ones() {
        val names = listOf(
            "context",
            "api",
            "contract",
            "entry-point",
            "technology",
            "fake"
        )
        val kinds = listOf(
            UnitKind.CONTEXT,
            UnitKind.API,
            UnitKind.CONTRACT,
            UnitKind.ENTRY_POINT,
            UnitKind.TECHNOLOGY,
            UnitKind.FAKE
        )

        assertEquals(kinds, names.map(String::toUnitKindOrNull))
        assertNull("library".toUnitKindOrNull())
        assertEquals(ProductKind.APP, "APP".toProductKindOrNull())
        assertEquals(ProductKind.PLATFORM_PLUGIN, "PLATFORM_PLUGIN".toProductKindOrNull())
        assertNull("server".toProductKindOrNull())
    }

    @Test
    fun GIVEN_missing_optional_product_context_WHEN_decoded_THEN_preserves_its_absence() {
        val context = UnitContext(
            UnitKind.CONTEXT,
            null,
            false,
            emptySet(),
            emptyList()
        )

        assertEquals(context, context.encode().toUnitContext())
    }

    @Test
    fun GIVEN_corrupt_or_incompatible_context_WHEN_decoded_THEN_fails_instead_of_guessing() {
        val encoded = UnitContext(
            UnitKind.CONTEXT,
            ProductKind.APP,
            false,
            emptySet(),
            emptyList()
        ).encode()

        assertFailsWith<IllegalArgumentException> { "not base64!".toUnitContext() }
        assertFailsWith<IllegalArgumentException> { "AAAAAw==".toUnitContext() }
        assertFailsWith<IllegalArgumentException> {
            replaceWireText(
                encoded,
                "context",
                "invalid"
            ).toUnitContext()
        }
        assertFailsWith<IllegalArgumentException> {
            replaceWireText(
                encoded,
                "APP",
                "BAD"
            ).toUnitContext()
        }
        val trailing = Base64.getDecoder().decode(encoded) + byteArrayOf(0)
        assertFailsWith<IllegalArgumentException> {
            Base64.getEncoder().encodeToString(trailing).toUnitContext()
        }
    }

    @Test
    fun GIVEN_invalid_compose_capability_WHEN_decoded_THEN_rejects_corrupt_context() {
        val context = UnitContext(
            UnitKind.CONTEXT,
            ProductKind.APP,
            false,
            emptySet(),
            listOf(
                UnitContext.Compilation(
                    "jvm:main",
                    emptyMap(),
                    false,
                    false,
                    emptySet()
                )
            )
        )

        assertFailsWith<IllegalStateException> {
            replaceWireText(
                context.encode(),
                "false",
                "bogus"
            ).toUnitContext()
        }
    }

    @Test
    fun GIVEN_unknown_ui_policy_values_WHEN_decoded_THEN_reports_the_corrupt_field() {
        val encoded = UnitContext(
            UnitKind.IMPL_UI,
            ProductKind.APP,
            false,
            emptySet(),
            emptyList()
        ).encode()

        val composition = assertFailsWith<IllegalArgumentException> {
            replaceWireText(
                encoded,
                "LOCAL",
                "OTHER"
            ).toUnitContext()
        }
        val access = assertFailsWith<IllegalArgumentException> {
            replaceWireText(
                encoded,
                "SIBLINGS",
                "INVALIDX"
            ).toUnitContext()
        }

        assertEquals(true, composition.message?.contains("UI composition"))
        assertEquals(true, access.message?.contains("implementation access"))
    }

    @Test
    fun GIVEN_corrupt_reason_entries_WHEN_decoded_THEN_rejects_each_malformed_policy() {
        val malformed = listOf(
            policyWire(-1),
            policyWire(
                1,
                " ",
                "reason"
            ),
            policyWire(
                1,
                "UnitDependency",
                " "
            ),
            policyWire(
                2,
                "UnitDependency",
                "first",
                "UnitDependency",
                "second"
            )
        )

        malformed.forEach { encoded ->
            assertFailsWith<IllegalArgumentException> { encoded.toUnitContext() }
        }
    }
}
