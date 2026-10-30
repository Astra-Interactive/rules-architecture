package ru.astrainteractive.klibs.rules.naming.detekt

import dev.detekt.api.Config
import dev.detekt.api.Entity
import dev.detekt.api.Finding
import dev.detekt.api.Rule
import dev.detekt.psi.hasAnnotation
import org.jetbrains.kotlin.lexer.KtTokens
import org.jetbrains.kotlin.psi.KtNamedFunction
import org.jetbrains.kotlin.psi.psiUtil.containingClassOrObject

/**
 * Reports a function whose first camel-case word is a verb the verb table forbids: `get`, `map`, `make`, `do`,
 * `handle` or `process`; the same letters later in the name are not banned. `getState()` is allowed in a view model,
 * its state accessor. Overrides, `operator` functions, DI `provide…` functions, `@Composable` and `@Test` functions
 * keep their names.
 */
internal class FunctionVerb(config: Config) : Rule(config, DESCRIPTION) {

    private fun isExempt(function: KtNamedFunction, name: String): Boolean {
        if (function.hasModifier(KtTokens.OVERRIDE_KEYWORD)) return true
        if (function.hasModifier(KtTokens.OPERATOR_KEYWORD)) return true
        if (name.startsWith(PROVIDE_PREFIX)) return true
        return EXEMPT_ANNOTATIONS.any(function::hasAnnotation)
    }

    private fun isViewModelState(function: KtNamedFunction, name: String): Boolean {
        val owner = function.containingClassOrObject?.name.orEmpty()
        return name == GET_STATE && owner.endsWith(VIEW_MODEL_SUFFIX)
    }

    private fun findMessage(function: KtNamedFunction, name: String): String? {
        val firstWord = name.split(WORD_START).firstOrNull(String::isNotEmpty)?.lowercase() ?: return null
        if (firstWord == GET && !isViewModelState(function, name)) {
            return "Name '$name' with a verb that says what happens instead of 'get': find, load, resolve, " +
                "build, compute or a val."
        }
        val forbidden = firstWord.takeIf { word -> word in FORBIDDEN_WORDS } ?: return null
        return "Name '$name' without '$forbidden': use a verb that says what happens, such as to<Type>, " +
            "on<Event>, build, update or save."
    }

    override fun visitNamedFunction(function: KtNamedFunction) {
        super.visitNamedFunction(function)
        val name = function.name ?: return
        if (isExempt(function, name)) return
        val message = findMessage(function, name) ?: return
        report(Finding(Entity.atName(function), message))
    }

    private companion object {
        const val GET = "get"
        const val GET_STATE = "getState"
        const val VIEW_MODEL_SUFFIX = "ViewModel"
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
