@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.gradle.linter.util

import ru.astrainteractive.klibs.rules.gradle.linter.model.ArchitectureFinding
import ru.astrainteractive.klibs.rules.gradle.linter.model.VersionCatalogSection
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

class VersionCatalogSectionExtTest {
    private fun section(
        vararg aliases: String,
        table: VersionCatalogSection.Table = VersionCatalogSection.Table.LIBRARIES,
        catalogName: String = CATALOG_NAME
    ): VersionCatalogSection {
        return VersionCatalogSection(
            catalogName = catalogName,
            table = table,
            aliases = aliases.toList()
        )
    }

    @Test
    fun GIVEN_library_alias_extending_another_WHEN_checked_THEN_reports_the_shorter_alias_once() {
        val findings = section("decompose-compose", "decompose-compose-experimental").findAliasPrefixFindings()

        val finding = findings.single()
        assertEquals("CatalogAliasPrefix", finding.rule)
        assertEquals("libs", finding.path)
        assertEquals("libraries: decompose-compose, decompose-compose-experimental", finding.subject)
        assertTrue(finding.message.contains("libs.decompose.compose"), finding.message)
        assertTrue(finding.message.contains("decompose-experimental-compose"), finding.message)
    }

    @Test
    fun GIVEN_aliases_sharing_characters_but_no_whole_segment_WHEN_checked_THEN_reports_nothing() {
        assertEquals(emptyList(), section("a-b", "a-bc").findAliasPrefixFindings())
        assertEquals(emptyList(), section("a-b", "a-c").findAliasPrefixFindings())
    }

    @Test
    fun GIVEN_aliases_whose_segments_are_out_of_order_WHEN_checked_THEN_reports_nothing() {
        assertEquals(emptyList(), section("b-a-c", "a-b").findAliasPrefixFindings())
    }

    @Test
    fun GIVEN_single_alias_or_no_alias_WHEN_checked_THEN_reports_nothing() {
        assertEquals(emptyList(), section("a").findAliasPrefixFindings())
        assertEquals(emptyList(), section().findAliasPrefixFindings())
    }

    @Test
    fun GIVEN_underscore_separated_alias_WHEN_checked_THEN_compares_its_segments_as_dashes() {
        val finding = section("a_b", "a.b.c").findAliasPrefixFindings().single()

        assertEquals("libraries: a-b, a-b-c", finding.subject)
    }

    @Test
    fun GIVEN_dot_separated_aliases_WHEN_checked_THEN_reports_the_same_finding_as_dashed_aliases() {
        val dotted = section("decompose.compose", "decompose.compose.experimental").findAliasPrefixFindings()
        val dashed = section("decompose-compose", "decompose-compose-experimental").findAliasPrefixFindings()

        assertEquals(dashed, dotted)
        assertEquals("libraries: decompose-compose, decompose-compose-experimental", dotted.single().subject)
    }

    @Test
    fun GIVEN_same_segments_spelled_with_different_separators_WHEN_checked_THEN_reports_one_pair() {
        val findings = section(
            "a-b",
            "a_b",
            "a-b-c"
        ).findAliasPrefixFindings()

        assertEquals(listOf("libraries: a-b, a-b-c"), findings.map(ArchitectureFinding::subject))
    }

    @Test
    fun GIVEN_plugin_alias_extending_another_WHEN_checked_THEN_reports_it_with_the_plugins_accessor() {
        val plugins = section(
            "kotlin",
            "kotlin-jvm",
            table = VersionCatalogSection.Table.PLUGINS
        )
        val finding = plugins.findAliasPrefixFindings().single()

        assertEquals("plugins: kotlin, kotlin-jvm", finding.subject)
        assertTrue(finding.message.contains("libs.plugins.kotlin"), finding.message)
        assertTrue(finding.message.contains("jvm-kotlin"), finding.message)
    }

    @Test
    fun GIVEN_bundle_alias_extending_another_WHEN_checked_THEN_reports_it_with_the_bundles_accessor() {
        val bundles = section(
            "ktor",
            "ktor-client",
            table = VersionCatalogSection.Table.BUNDLES
        )
        val finding = bundles.findAliasPrefixFindings().single()

        assertEquals("bundles: ktor, ktor-client", finding.subject)
        assertTrue(finding.message.contains("libs.bundles.ktor"), finding.message)
    }

    @Test
    fun GIVEN_chain_of_prefixes_WHEN_checked_THEN_reports_every_pair_in_alias_order() {
        val findings = section(
            "a-b-c",
            "a",
            "a-b"
        ).findAliasPrefixFindings()

        assertEquals(
            listOf(
                "libraries: a, a-b",
                "libraries: a, a-b-c",
                "libraries: a-b, a-b-c"
            ),
            findings.map(ArchitectureFinding::subject)
        )
    }

    @Test
    fun GIVEN_catalog_with_another_name_WHEN_checked_THEN_names_its_accessor_after_that_catalog() {
        val tools = section(
            "x",
            "x-y",
            catalogName = "tools"
        )
        val finding = tools.findAliasPrefixFindings().single()

        assertEquals("tools", finding.path)
        assertTrue(finding.message.contains("tools.x"), finding.message)
    }

    private companion object {
        const val CATALOG_NAME = "libs"
    }
}
