package ru.astrainteractive.klibs.rules.detekt.time

import dev.detekt.api.Config
import dev.detekt.api.Entity
import dev.detekt.api.Finding
import dev.detekt.api.Rule
import dev.detekt.psi.hasAnnotation
import org.jetbrains.kotlin.lexer.KtTokens
import org.jetbrains.kotlin.psi.KtCallableDeclaration
import org.jetbrains.kotlin.psi.KtClassOrObject
import org.jetbrains.kotlin.psi.KtConstantExpression
import org.jetbrains.kotlin.psi.KtNullableType
import org.jetbrains.kotlin.psi.KtParameter
import org.jetbrains.kotlin.psi.KtProperty
import org.jetbrains.kotlin.psi.KtStringTemplateExpression
import org.jetbrains.kotlin.psi.KtUserType
import org.jetbrains.kotlin.psi.psiUtil.getStrictParentOfType

/**
 * Reports a property or parameter whose name says it holds a moment or a duration (`createdAt`, `timeoutMs`,
 * `delay`, `CARD_IN_MS`) while its type is a number or a string, declared or inferred from a literal
 * (`const val CARD_IN_MS = 400`). The name is split into camel-case and underscore words: a time word (`delay`,
 * `timestamp`, ...) is judged as the whole name or its last word, a unit (`Ms`, `SECONDS`, `At`, ...) as the last
 * word after another.
 *
 * Works on names, so it is a heuristic. A `@Serializable` class is exempt, because a wire format is a boundary
 * where the primitive is allowed; so is an `override`, whose signature is dictated by its supertype.
 */
internal class TimeAsPrimitive(config: Config) : Rule(config, DESCRIPTION) {

    private fun literalTypeName(literal: KtConstantExpression): String? {
        val text = literal.text
        return when {
            INTEGER_LITERAL.matches(text) -> if (text.endsWith('L')) "Long" else "Int"
            FLOAT_LITERAL.matches(text) -> if (text.endsWith('f') || text.endsWith('F')) "Float" else "Double"
            else -> null
        }
    }

    private fun inferredTypeName(declaration: KtCallableDeclaration): String? {
        return when (val initializer = (declaration as? KtProperty)?.initializer) {
            is KtStringTemplateExpression -> "String"
            is KtConstantExpression -> literalTypeName(initializer)
            else -> null
        }
    }

    private fun primitiveTypeName(declaration: KtCallableDeclaration): String? {
        val typeReference = declaration.typeReference ?: return inferredTypeName(declaration)
        val typeElement = typeReference.typeElement
        val userType = when (typeElement) {
            is KtNullableType -> typeElement.innerType as? KtUserType
            is KtUserType -> typeElement
            else -> null
        }
        return userType?.referencedName?.takeIf { name -> name in PRIMITIVE_TYPES }
    }

    private fun isTimeName(name: String): Boolean {
        val words = name.split(WORD_BOUNDARY)
            .filter { word -> word.isNotEmpty() }
            .map { word -> word.lowercase() }
        val lastWord = words.lastOrNull() ?: return false
        if (lastWord in TIME_WORDS) return true
        return words.size > 1 && lastWord in TIME_UNITS
    }

    private fun isOverride(declaration: KtCallableDeclaration): Boolean {
        if (declaration.hasModifier(KtTokens.OVERRIDE_KEYWORD)) return true
        val ownerFunction = (declaration as? KtParameter)?.ownerFunction ?: return false
        return ownerFunction.hasModifier(KtTokens.OVERRIDE_KEYWORD)
    }

    private fun isExempt(declaration: KtCallableDeclaration): Boolean {
        if (isOverride(declaration)) return true
        val owner = declaration.getStrictParentOfType<KtClassOrObject>() ?: return false
        return owner.hasAnnotation(SERIALIZABLE)
    }

    private fun check(declaration: KtCallableDeclaration) {
        val name = declaration.name ?: return
        if (!isTimeName(name)) return
        val typeName = primitiveTypeName(declaration) ?: return
        if (isExempt(declaration)) return
        val message = "'$name' is a time value typed $typeName: use kotlin.time.Instant for a moment " +
            "or kotlin.time.Duration for an interval."
        report(Finding(Entity.from(declaration), message))
    }

    override fun visitProperty(property: KtProperty) {
        super.visitProperty(property)
        check(property)
    }

    override fun visitParameter(parameter: KtParameter) {
        super.visitParameter(parameter)
        check(parameter)
    }

    private companion object {
        const val SERIALIZABLE = "Serializable"

        @JvmField
        val PRIMITIVE_TYPES = setOf("Long", "Int", "String", "Double", "Float")

        @JvmField
        val TIME_WORDS = setOf("cooldown", "date", "delay", "duration", "interval", "time", "timeout", "timestamp")

        @JvmField
        val TIME_UNITS = setOf("at", "days", "hours", "millis", "minutes", "ms", "nanos", "seconds", "secs")

        @JvmField
        val WORD_BOUNDARY = Regex("_+|(?<=[a-z0-9])(?=[A-Z])|(?<=[A-Z])(?=[A-Z][a-z])")

        @JvmField
        val INTEGER_LITERAL = Regex("""(0[xXbB])?[0-9a-fA-F_]+[uU]?[lL]?""")

        @JvmField
        val FLOAT_LITERAL = Regex("""[0-9_]*\.?[0-9_]+([eE][+-]?[0-9_]+)?[fF]?""")
        const val DESCRIPTION = "Moments are kotlin.time.Instant and intervals kotlin.time.Duration, never primitives."
    }
}
