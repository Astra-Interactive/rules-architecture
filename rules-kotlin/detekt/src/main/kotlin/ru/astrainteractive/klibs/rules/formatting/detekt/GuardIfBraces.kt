package ru.astrainteractive.klibs.rules.formatting.detekt

import dev.detekt.api.Config
import dev.detekt.api.Entity
import dev.detekt.api.Finding
import dev.detekt.api.Rule
import org.jetbrains.kotlin.psi.KtBlockExpression
import org.jetbrains.kotlin.psi.KtIfExpression
import org.jetbrains.kotlin.psi.KtReturnExpression

/**
 * Reports an `if` without `else` whose whole body is one `return` in braces, when `if (condition) return …` fits
 * on one line within the project's `style.MaxLineLength.maxLineLength`: a guard is one line without braces.
 */
internal class GuardIfBraces(config: Config) : Rule(config, DESCRIPTION) {
    private val maxLineLength: Int by lazy {
        val root = generateSequence(config) { current -> current.parent?.takeUnless { parent -> parent === current } }
            .last()
        root.subConfig("style").subConfig("MaxLineLength").valueOrDefault("maxLineLength", DEFAULT_MAX_LINE_LENGTH)
    }

    private fun columnOf(expression: KtIfExpression): Int {
        val fileText = expression.containingFile.text
        val start = expression.textRange.startOffset
        val lineStart = fileText.lastIndexOf('\n', start - 1) + 1
        return start - lineStart
    }

    override fun visitIfExpression(expression: KtIfExpression) {
        super.visitIfExpression(expression)
        if (expression.`else` != null) return
        val body = expression.then as? KtBlockExpression ?: return
        val statement = body.statements.singleOrNull() as? KtReturnExpression ?: return
        val condition = expression.condition?.text ?: return
        val oneLine = "if ($condition) ${statement.text}"
        if ('\n' in oneLine || columnOf(expression) + oneLine.length > maxLineLength) return
        val message = "Write the guard on one line without braces: '$oneLine'."
        report(Finding(Entity.from(expression), message))
    }

    private companion object {
        const val DEFAULT_MAX_LINE_LENGTH = 120
        const val DESCRIPTION = "An if whose whole body is one return is one line without braces when it fits."
    }
}
