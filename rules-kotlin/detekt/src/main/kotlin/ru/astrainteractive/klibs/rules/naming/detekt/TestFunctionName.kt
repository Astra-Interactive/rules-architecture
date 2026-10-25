package ru.astrainteractive.klibs.rules.naming.detekt

import dev.detekt.api.Config
import dev.detekt.api.Entity
import dev.detekt.api.Finding
import dev.detekt.api.Rule
import dev.detekt.psi.hasAnnotation
import org.jetbrains.kotlin.psi.KtNamedFunction

/**
 * Reports a `@Test` function not named `GIVEN_<state>_WHEN_<action>_THEN_<outcome>`, with every part between
 * the keywords in lower-case `snake_case`.
 */
internal class TestFunctionName(config: Config) : Rule(config, DESCRIPTION) {

    override fun visitNamedFunction(function: KtNamedFunction) {
        super.visitNamedFunction(function)
        if (!function.hasAnnotation(TEST_ANNOTATION)) return
        val name = function.name ?: return
        if (TEST_NAME.matches(name)) return
        val message = "Rename '$name' to GIVEN_<state>_WHEN_<action>_THEN_<outcome> in snake_case."
        report(Finding(Entity.from(function), message))
    }

    private companion object {
        const val TEST_ANNOTATION = "Test"

        @JvmField
        val TEST_NAME = Regex("GIVEN(_[a-z0-9]+)+_WHEN(_[a-z0-9]+)+_THEN(_[a-z0-9]+)+")
        const val DESCRIPTION = "Test names follow GIVEN_<state>_WHEN_<action>_THEN_<outcome>."
    }
}
