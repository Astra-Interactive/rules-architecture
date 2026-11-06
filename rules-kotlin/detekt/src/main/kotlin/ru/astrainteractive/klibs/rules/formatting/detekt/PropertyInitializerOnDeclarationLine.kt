package ru.astrainteractive.klibs.rules.formatting.detekt

import dev.detekt.api.Config
import dev.detekt.api.Entity
import dev.detekt.api.Finding
import dev.detekt.api.Rule
import org.jetbrains.kotlin.lexer.KtTokens
import org.jetbrains.kotlin.psi.KtDestructuringDeclaration
import org.jetbrains.kotlin.psi.KtExpression
import org.jetbrains.kotlin.psi.KtProperty
import org.jetbrains.kotlin.psi.psiUtil.siblings

/**
 * Reports a property or a destructuring declaration (`val (id, total) =`) whose initializer starts on a later line
 * than its `=`: a line break between `=` and the first token of the initializer, a comment that ends the line of
 * `=` included. Only where the initializer starts is judged: one that starts on the line of `=` and then spans
 * several lines passes. A delegate (`by lazy {`), an accessor body (`get() = …`) and a parameter's default value
 * are not initializers.
 */
internal class PropertyInitializerOnDeclarationLine(config: Config) : Rule(config, DESCRIPTION) {

    private fun startsAfterEqualsLine(initializer: KtExpression?): Boolean {
        if (initializer == null) return false
        return initializer.siblings(forward = false, withItself = false)
            .takeWhile { sibling -> sibling.node.elementType != KtTokens.EQ }
            .any { sibling -> '\n' in sibling.text }
    }

    private fun reportWrappedInitializer(name: String?, entity: Entity) {
        val message = "The initializer of '$name' starts on the line after its '=': join the declaration into one " +
            "line when it fits MaxLineLength; otherwise start the initializer on the '=' line and wrap inside it: " +
            "arguments one per line, a chain one call per line with the receiver on the '=' line, a long receiver " +
            "named as its own val."
        report(Finding(entity, message))
    }

    override fun visitProperty(property: KtProperty) {
        super.visitProperty(property)
        if (!startsAfterEqualsLine(property.initializer)) return
        reportWrappedInitializer(property.name, Entity.atName(property))
    }

    override fun visitDestructuringDeclaration(multiDeclaration: KtDestructuringDeclaration) {
        super.visitDestructuringDeclaration(multiDeclaration)
        if (!startsAfterEqualsLine(multiDeclaration.initializer)) return
        val names = multiDeclaration.entries.joinToString(
            separator = ", ",
            prefix = "(",
            postfix = ")",
            transform = KtExpression::getText
        )
        reportWrappedInitializer(names, Entity.from(multiDeclaration))
    }

    private companion object {
        const val DESCRIPTION = "A property initializer starts on the line of its =."
    }
}
