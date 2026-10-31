package ru.astrainteractive.klibs.rules.coroutine.detekt

import dev.detekt.api.Config
import dev.detekt.api.Entity
import dev.detekt.api.Finding
import dev.detekt.api.Rule
import org.jetbrains.kotlin.psi.KtCallExpression
import org.jetbrains.kotlin.psi.KtDotQualifiedExpression
import org.jetbrains.kotlin.psi.KtLambdaExpression
import org.jetbrains.kotlin.psi.psiUtil.parents
import ru.astrainteractive.klibs.rules.detekt.util.findCallName

/**
 * Reports one-off events (`viewModel.events`, `…Events`) collected by the UI off `Dispatchers.Main.immediate`:
 * inside `LaunchedEffect` without `withContext(Dispatchers.Main.immediate)`, and elsewhere without
 * `repeatOnLifecycle`. The UI handles an event at once, inside its lifecycle.
 */
internal class EventCollection(config: Config) : Rule(config, DESCRIPTION) {

    private fun isEventFlow(call: KtCallExpression): Boolean {
        val chain = call.parent as? KtDotQualifiedExpression ?: return false
        if (chain.selectorExpression != call) return false
        val receiver = chain.receiverExpression.text.substringAfterLast('.')
        return receiver == EVENTS || receiver.endsWith(EVENTS_SUFFIX)
    }

    private fun enclosingCallNames(call: KtCallExpression): List<String?> {
        return call.parents.filterIsInstance<KtLambdaExpression>().map(KtLambdaExpression::findCallName).toList()
    }

    private fun isOnMainImmediate(call: KtCallExpression): Boolean {
        return call.parents
            .filterIsInstance<KtCallExpression>()
            .filter { parent -> parent.calleeExpression?.text == WITH_CONTEXT }
            .any { parent -> parent.valueArguments.any { argument -> MAIN_IMMEDIATE in argument.text } }
    }

    private fun findMessage(call: KtCallExpression): String? {
        val callNames = enclosingCallNames(call)
        if (LAUNCHED_EFFECT in callNames) {
            if (isOnMainImmediate(call)) return null
            return "Collect these events inside withContext(Dispatchers.Main.immediate) in the LaunchedEffect: the " +
                "UI handles an event at once, on the frame it arrives."
        }
        if (REPEAT_ON_LIFECYCLE in callNames) return null
        return "Collect these events inside repeatOnLifecycle: the UI collects one-off events only while it is " +
            "started, on Dispatchers.Main.immediate."
    }

    override fun visitCallExpression(expression: KtCallExpression) {
        super.visitCallExpression(expression)
        if (expression.calleeExpression?.text !in COLLECT_CALLS || !isEventFlow(expression)) return
        val message = findMessage(expression) ?: return
        report(Finding(Entity.from(expression), message))
    }

    private companion object {
        const val EVENTS = "events"
        const val EVENTS_SUFFIX = "Events"
        const val WITH_CONTEXT = "withContext"
        const val MAIN_IMMEDIATE = "Dispatchers.Main.immediate"
        const val LAUNCHED_EFFECT = "LaunchedEffect"
        const val REPEAT_ON_LIFECYCLE = "repeatOnLifecycle"

        @JvmField
        val COLLECT_CALLS = setOf("collect", "collectLatest")
        const val DESCRIPTION = "The UI collects one-off events on Dispatchers.Main.immediate, inside its lifecycle."
    }
}
