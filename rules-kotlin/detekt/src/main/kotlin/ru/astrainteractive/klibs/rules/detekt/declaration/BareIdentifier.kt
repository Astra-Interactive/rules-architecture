package ru.astrainteractive.klibs.rules.detekt.declaration

import dev.detekt.api.Config
import dev.detekt.api.Entity
import dev.detekt.api.Finding
import dev.detekt.api.Rule
import dev.detekt.psi.hasAnnotation
import org.jetbrains.kotlin.lexer.KtTokens
import org.jetbrains.kotlin.name.Name
import org.jetbrains.kotlin.psi.KtCallableDeclaration
import org.jetbrains.kotlin.psi.KtClass
import org.jetbrains.kotlin.psi.KtClassOrObject
import org.jetbrains.kotlin.psi.KtNullableType
import org.jetbrains.kotlin.psi.KtParameter
import org.jetbrains.kotlin.psi.KtProperty
import org.jetbrains.kotlin.psi.KtUserType
import org.jetbrains.kotlin.psi.psiUtil.getStrictParentOfType

/**
 * Reports a bare identifier: a property or constructor property named `id` or `…Id` typed as a string, a number
 * or a UUID, and a class named `…Id` that is not a value class. A `@Serializable` class, a technology package
 * (`exposed`, `room`, `table`, `entity`, `contract`, …) and an `override` keep the primitive, because there the
 * format or the supertype dictates it.
 */
internal class BareIdentifier(config: Config) : Rule(config, DESCRIPTION) {

    private fun primitiveTypeName(declaration: KtCallableDeclaration): String? {
        val typeElement = declaration.typeReference?.typeElement
        val userType = when (typeElement) {
            is KtNullableType -> typeElement.innerType as? KtUserType
            is KtUserType -> typeElement
            else -> null
        }
        return userType?.referencedName?.takeIf { name -> name in PRIMITIVE_TYPES }
    }

    private fun isTechnologyPackage(declaration: KtCallableDeclaration): Boolean {
        val segments = declaration.containingKtFile.packageFqName.pathSegments().map(Name::asString)
        return segments.any { segment -> segment in TECHNOLOGY_WORDS }
    }

    private fun isExempt(declaration: KtCallableDeclaration, owner: KtClassOrObject): Boolean {
        if (declaration.hasModifier(KtTokens.OVERRIDE_KEYWORD)) return true
        if (owner.hasAnnotation(SERIALIZABLE)) return true
        return isTechnologyPackage(declaration)
    }

    private fun check(declaration: KtCallableDeclaration) {
        val name = declaration.name ?: return
        if (!IDENTIFIER_NAME.matches(name)) return
        val typeName = primitiveTypeName(declaration) ?: return
        val owner = declaration.getStrictParentOfType<KtClassOrObject>() ?: return
        if (isExempt(declaration, owner)) return
        val message = "'$name' is an identifier typed $typeName: give it a typed identifier, a value class such " +
            "as PlayerId, instead of the bare primitive."
        report(Finding(Entity.from(declaration), message))
    }

    override fun visitProperty(property: KtProperty) {
        super.visitProperty(property)
        if (!property.isLocal) check(property)
    }

    override fun visitParameter(parameter: KtParameter) {
        super.visitParameter(parameter)
        if (parameter.hasValOrVar()) check(parameter)
    }

    override fun visitClass(klass: KtClass) {
        super.visitClass(klass)
        val name = klass.name ?: return
        if (!name.endsWith(ID_SUFFIX) || klass.isInterface() || klass.isEnum()) return
        if (klass.isValue() || klass.isInline()) return
        val message = "'$name' is an identifier: make it a @JvmInline value class over the primitive."
        report(Finding(Entity.atName(klass), message))
    }

    private companion object {
        const val SERIALIZABLE = "Serializable"
        const val ID_SUFFIX = "Id"

        @JvmField
        val IDENTIFIER_NAME = Regex("id|.*[a-z0-9]Id")

        @JvmField
        val PRIMITIVE_TYPES = setOf(
            "String",
            "Int",
            "Long",
            "UUID",
            "Uuid"
        )

        @JvmField
        val TECHNOLOGY_WORDS = setOf(
            "exposed",
            "room",
            "ktor",
            "krate",
            "sqldelight",
            "datastore",
            "retrofit",
            "okhttp",
            "cache4k",
            "file",
            "memory",
            "table",
            "entity",
            "contract"
        )
        const val DESCRIPTION = "An entity carries a typed identifier, a value class, never a bare string or number."
    }
}
