package ru.astrainteractive.klibs.rules.ordering.detekt

import dev.detekt.api.Config
import dev.detekt.api.Entity
import dev.detekt.api.Finding
import dev.detekt.api.Rule
import org.jetbrains.kotlin.psi.KtCallExpression
import org.jetbrains.kotlin.psi.KtClassBody
import org.jetbrains.kotlin.psi.KtDeclaration
import org.jetbrains.kotlin.psi.KtExpression
import org.jetbrains.kotlin.psi.KtFile
import org.jetbrains.kotlin.psi.KtNameReferenceExpression
import org.jetbrains.kotlin.psi.KtNamedFunction
import org.jetbrains.kotlin.psi.KtQualifiedExpression
import org.jetbrains.kotlin.psi.KtThisExpression
import org.jetbrains.kotlin.psi.psiUtil.collectDescendantsOfType

/**
 * Reports a function that calls a function of the same class or file declared below it.
 *
 * Calls are matched by name, and a call on another receiver counts only when a sibling extension has that name.
 * Recursion and mutual recursion, through any number of functions, are skipped, because no order satisfies them,
 * and so is a name with an overload declared above the caller. Property initializers are not checked: properties stay
 * on top.
 */
internal class CalleeBeforeCaller(config: Config) : Rule(config, DESCRIPTION) {

    private fun receiverOf(call: KtCallExpression): KtExpression? {
        val qualified = call.parent as? KtQualifiedExpression ?: return null
        if (qualified.selectorExpression != call) return null
        return qualified.receiverExpression
    }

    private fun calledNames(function: KtNamedFunction, extensionNames: Set<String>): Set<String> {
        return function.collectDescendantsOfType<KtCallExpression>()
            .mapNotNull { call ->
                val callee = call.calleeExpression as? KtNameReferenceExpression ?: return@mapNotNull null
                val name = callee.getReferencedName()
                val receiver = receiverOf(call)
                val isSiblingCall = receiver == null || receiver is KtThisExpression || name in extensionNames
                name.takeIf { isSiblingCall }
            }
            .toSet()
    }

    private fun checkScope(declarations: List<KtDeclaration>) {
        val functions = declarations.filterIsInstance<KtNamedFunction>()
        val extensionNames = functions
            .filter { function -> function.receiverTypeReference != null }
            .mapNotNull(KtNamedFunction::getName)
            .toSet()
        val firstIndexByName = functions
            .withIndex()
            .reversed()
            .associateBy(
                keySelector = { indexed -> indexed.value.name },
                valueTransform = IndexedValue<KtNamedFunction>::index
            )
        val calls = functions.map { function -> calledNames(function, extensionNames) }

        val callsByName = functions.indices.groupBy(
            keySelector = { index -> functions[index].name },
            valueTransform = { index -> calls[index] }
        ).mapValues { (_, called) -> called.flatten().toSet() }

        fun callsBack(calleeName: String, callerName: String): Boolean {
            val visited = mutableSetOf(calleeName)
            val queue = ArrayDeque(listOf(calleeName))
            while (queue.isNotEmpty()) {
                val next = callsByName[queue.removeFirst()].orEmpty()
                if (callerName in next) return true
                next.filter(visited::add).forEach(queue::addLast)
            }
            return false
        }

        functions.forEachIndexed { callerIndex, caller ->
            val callerName = caller.name ?: return@forEachIndexed
            calls[callerIndex]
                .filter { calleeName -> calleeName != callerName }
                .filter { calleeName -> (firstIndexByName[calleeName] ?: -1) > callerIndex }
                .filterNot { calleeName -> callsBack(calleeName, callerName) }
                .forEach { calleeName ->
                    val message = "'$callerName' calls '$calleeName', which is declared below it: " +
                        "move '$calleeName' above '$callerName'."
                    report(Finding(Entity.from(caller), message))
                }
        }
    }

    override fun visitKtFile(file: KtFile) {
        super.visitKtFile(file)
        checkScope(file.declarations)
    }

    override fun visitClassBody(classBody: KtClassBody) {
        super.visitClassBody(classBody)
        checkScope(classBody.declarations)
    }

    private companion object {
        const val DESCRIPTION = "A function is declared above the functions that call it."
    }
}
