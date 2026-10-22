package ru.astrainteractive.klibs.rules.detekt.naming

import dev.detekt.api.Config
import dev.detekt.api.Entity
import dev.detekt.api.Finding
import dev.detekt.api.Rule
import dev.detekt.psi.hasAnnotation
import org.jetbrains.kotlin.lexer.KtTokens
import org.jetbrains.kotlin.psi.KtNamedFunction

/**
 * Reports a function named with a verb the verb table forbids: a `get` prefix (`getState` aside), and the words
 * `map`, `make`, `do`, `handle` and `process` as a camel-case word of the name. Overrides, `operator` functions,
 * DI `provide…` functions, `@Composable` and `@Test` functions keep their names.
 */
internal class FunctionVerb(config: Config) : Rule(config, DESCRIPTION) {

    private fun isExempt(function: KtNamedFunction, name: String): Boolean {
        if (function.hasModifier(KtTokens.OVERRIDE_KEYWORD)) return true
        if (function.hasModifier(KtTokens.OPERATOR_KEYWORD)) return true
        if (name.startsWith(PROVIDE_PREFIX)) return true
        return EXEMPT_ANNOTATIONS.any { annotation -> function.hasAnnotation(annotation) }
    }

    private fun findMessage(name: String): String? {
        val words = name.split(WORD_START).filter { word -> word.isNotEmpty() }.map { word -> word.lowercase() }
        if (words.firstOrNull() == GET && name != GET_STATE) {
            return "Name '$name' with a verb that says what happens instead of 'get': find, load, resolve, " +
                "build, compute or a val."
        }
        val forbidden = words.firstOrNull { word -> word in FORBIDDEN_WORDS } ?: return null
        return "Name '$name' without '$forbidden': use a verb that says what happens, such as to<Type>, " +
            "on<Event>, build, update or save."
    }

    override fun visitNamedFunction(function: KtNamedFunction) {
        super.visitNamedFunction(function)
        val name = function.name ?: return
        if (isExempt(function, name)) return
        val message = findMessage(name) ?: return
        report(Finding(Entity.atName(function), message))
    }

    private companion object {
        const val GET = "get"
        const val GET_STATE = "getState"
        const val PROVIDE_PREFIX = "provide"

        @JvmField
        val WORD_START = Regex("(?=[A-Z])|_")

        @JvmField
        val FORBIDDEN_WORDS = setOf(
            "map",
            "make",
            "do",
            "handle",
            "process"
        )

        @JvmField
        val EXEMPT_ANNOTATIONS = listOf("Composable", "Test")
        const val DESCRIPTION = "A function is named with a verb from the verb table, never get, map, make, do, " +
            "handle or process."
    }
}
