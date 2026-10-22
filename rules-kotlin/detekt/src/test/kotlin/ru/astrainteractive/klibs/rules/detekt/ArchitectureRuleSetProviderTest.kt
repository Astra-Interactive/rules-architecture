@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.detekt

import dev.detekt.api.RuleName
import org.junit.jupiter.api.Test
import java.io.BufferedReader
import kotlin.test.assertEquals

internal class ArchitectureRuleSetProviderTest {
    private val ruleLine = Regex("^ {2}([A-Za-z]+):$")

    private fun readActiveRulesFromConfig(): Set<String> {
        val stream = checkNotNull(javaClass.classLoader.getResourceAsStream("config/config.yml")) {
            "config/config.yml is missing from the rule set jar"
        }
        val lines = stream.bufferedReader().use(BufferedReader::readLines)
        return lines
            .windowed(size = 2)
            .mapNotNull { window ->
                val ruleName = ruleLine.find(window.first())?.groupValues?.get(1) ?: return@mapNotNull null
                ruleName.takeIf { window.last().trim() == "active: true" }
            }
            .toSet()
    }

    @Test
    fun GIVEN_provider_WHEN_instance_created_THEN_uses_architecture_rule_set_id() {
        val provider = ArchitectureRuleSetProvider()

        assertEquals("architecture", provider.instance().id.value)
    }

    @Test
    fun GIVEN_bundled_config_WHEN_compared_with_provider_THEN_every_rule_is_listed_active() {
        val providedRules = ArchitectureRuleSetProvider().instance().rules.keys
            .map(RuleName::value)
            .toSet()

        assertEquals(providedRules, readActiveRulesFromConfig())
    }
}
