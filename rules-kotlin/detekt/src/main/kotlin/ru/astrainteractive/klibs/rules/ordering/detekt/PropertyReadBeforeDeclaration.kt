package ru.astrainteractive.klibs.rules.ordering.detekt

import com.intellij.psi.PsiElement
import dev.detekt.api.Config
import dev.detekt.api.Entity
import dev.detekt.api.Finding
import dev.detekt.api.RequiresAnalysisApi
import dev.detekt.api.Rule
import org.jetbrains.kotlin.analysis.api.analyze
import org.jetbrains.kotlin.idea.references.mainReference
import org.jetbrains.kotlin.lexer.KtTokens
import org.jetbrains.kotlin.psi.KtAnonymousInitializer
import org.jetbrains.kotlin.psi.KtCallableReferenceExpression
import org.jetbrains.kotlin.psi.KtClassOrObject
import org.jetbrains.kotlin.psi.KtDeclaration
import org.jetbrains.kotlin.psi.KtDotQualifiedExpression
import org.jetbrains.kotlin.psi.KtExpression
import org.jetbrains.kotlin.psi.KtFunction
import org.jetbrains.kotlin.psi.KtLambdaExpression
import org.jetbrains.kotlin.psi.KtNameReferenceExpression
import org.jetbrains.kotlin.psi.KtObjectLiteralExpression
import org.jetbrains.kotlin.psi.KtProperty
import org.jetbrains.kotlin.psi.KtThisExpression
import org.jetbrains.kotlin.psi.psiUtil.collectDescendantsOfType

/**
 * Reports a property of a class or object read while an instance is being constructed, before the property is
 * declared: in a property initializer, a property delegate or an `init` block, the property still holds `null` or
 * zero. Code that runs later reads it safely and is not judged: a function, a getter, a lambda (`by lazy { }`), an
 * object literal and a callable reference. A `const val` is inlined by the compiler and can be read anywhere. The
 * read is matched to the property through the Analysis API, so a local or a parameter of the same name is not
 * mistaken for it.
 */
internal class PropertyReadBeforeDeclaration(config: Config) : Rule(config, DESCRIPTION), RequiresAnalysisApi {

    private fun constructionCode(member: KtDeclaration): List<KtExpression> {
        return when (member) {
            is KtProperty -> listOfNotNull(member.initializer, member.delegateExpression)
            is KtAnonymousInitializer -> listOfNotNull(member.body)
            else -> emptyList()
        }
    }

    private fun runsLater(element: PsiElement): Boolean {
        return LATER_CODE.any { type -> type.isInstance(element) }
    }

    private fun readsAnotherInstance(reference: KtNameReferenceExpression): Boolean {
        val qualified = reference.parent as? KtDotQualifiedExpression ?: return false
        return qualified.selectorExpression == reference && qualified.receiverExpression !is KtThisExpression
    }

    private fun findReads(code: KtExpression, notYetDeclared: Set<KtProperty>): List<KtNameReferenceExpression> {
        val references = code.collectDescendantsOfType<KtNameReferenceExpression>(
            canGoInside = { element -> !runsLater(element) }
        )
        return references
            .filterNot(::readsAnotherInstance)
            .filter { reference ->
                val target = analyze(reference) { reference.mainReference.resolveToSymbol()?.psi }
                target in notYetDeclared
            }
    }

    override fun visitClassOrObject(classOrObject: KtClassOrObject) {
        super.visitClassOrObject(classOrObject)
        val members = classOrObject.body?.declarations.orEmpty().filterNot { member -> member is KtClassOrObject }
        members.forEachIndexed { index, member ->
            val notYetDeclared = members.drop(index)
                .filterIsInstance<KtProperty>()
                .filterNot { property -> property.hasModifier(KtTokens.CONST_KEYWORD) }
                .toSet()
            constructionCode(member)
                .flatMap { code -> findReads(code, notYetDeclared) }
                .forEach { reference ->
                    val name = reference.getReferencedName()
                    val message = "'$name' is read before it is declared, while the object is still being " +
                        "constructed: declare '$name' above this line or read it in code that runs later."
                    report(Finding(Entity.from(reference), message))
                }
        }
    }

    private companion object {
        const val DESCRIPTION = "A property is not read during construction before it is declared."

        @JvmField
        val LATER_CODE = listOf(
            KtLambdaExpression::class.java,
            KtFunction::class.java,
            KtObjectLiteralExpression::class.java,
            KtCallableReferenceExpression::class.java
        )
    }
}
