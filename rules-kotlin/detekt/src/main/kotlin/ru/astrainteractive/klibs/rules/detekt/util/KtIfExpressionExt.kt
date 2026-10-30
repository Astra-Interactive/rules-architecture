package ru.astrainteractive.klibs.rules.detekt.util

import org.jetbrains.kotlin.psi.KtBlockExpression
import org.jetbrains.kotlin.psi.KtIfExpression
import org.jetbrains.kotlin.psi.KtThrowExpression

/**
 * Whether the `then` branch only throws, written bare or as a block of one statement: `if (error is X) throw error`.
 */
internal fun KtIfExpression.hasThrowBranch(): Boolean {
    val branch = then
    val statement = (branch as? KtBlockExpression)?.statements?.singleOrNull() ?: branch
    return statement is KtThrowExpression
}
