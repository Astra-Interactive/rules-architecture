package ru.astrainteractive.klibs.rules.detekt.time

import dev.detekt.api.Config
import dev.detekt.api.Entity
import dev.detekt.api.Finding
import dev.detekt.api.Rule
import dev.detekt.psi.hasAnnotation
import org.jetbrains.kotlin.lexer.KtTokens
import org.jetbrains.kotlin.psi.KtCallableDeclaration
import org.jetbrains.kotlin.psi.KtClassOrObject
import org.jetbrains.kotlin.psi.KtNullableType
import org.jetbrains.kotlin.psi.KtParameter
import org.jetbrains.kotlin.psi.KtProperty
import org.jetbrains.kotlin.psi.KtUserType
import org.jetbrains.kotlin.psi.psiUtil.getStrictParentOfType

/**
 * Reports a property or parameter whose name says it holds a moment or a duration (`createdAt`, `timeoutMs`,
 * `delay`) while its declared type is a number or a string.
 *
 * Works on names, so it is a heuristic. A `@Serializable` class is exempt, because a wire format is a boundary
 * where the primitive is allowed; so is an `override`, whose signature is dictated by its supertype.
 */
internal class TimeAsPrimitive(config: Config) : Rule(config, DESCRIPTION) {

    private fun primitiveTypeName(declaration: KtCallableDeclaration): String? {
        val typeElement = declaration.typeReference?.typeElement
        val userType = when (typeElement) {
            is KtNullableType -> typeElement.innerType as? KtUserType
            is KtUserType -> typeElement
            else -> null
        }
        return userType?.referencedName?.takeIf { name -> name in PRIMITIVE_TYPES }
    }

    private fun isTimeName(name: String): Boolean = TIME_WORD.matches(name) || TIME_SUFFIX.containsMatchIn(name)

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
        val TIME_WORD = Regex("(?i)timeout|delay|timestamp|duration|interval|cooldown|date|time")

        @JvmField
        val TIME_SUFFIX = Regex(
            "[a-z0-9](At|Millis|Ms|Nanos|Seconds|Secs|Minutes|Hours|Days|Timestamp|Time|Date|Duration|Timeout|Delay|" +
                "Interval|Cooldown)$"
        )
        const val DESCRIPTION = "Moments are kotlin.time.Instant and intervals kotlin.time.Duration, never primitives."
    }
}
