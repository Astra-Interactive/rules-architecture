package ru.astrainteractive.klibs.rules.time.detekt

import dev.detekt.api.Config
import dev.detekt.api.Entity
import dev.detekt.api.Finding
import dev.detekt.api.Rule
import org.jetbrains.kotlin.psi.KtCallExpression
import org.jetbrains.kotlin.psi.KtDotQualifiedExpression

/**
 * Reports `System.currentTimeMillis()` and `System.nanoTime()`: a moment is a `kotlin.time.Instant` from a clock.
 */
internal class SystemTimeCall(config: Config) : Rule(config, DESCRIPTION) {

    override fun visitDotQualifiedExpression(expression: KtDotQualifiedExpression) {
        super.visitDotQualifiedExpression(expression)
        val receiver = expression.receiverExpression.text
        if (receiver != SYSTEM && receiver != QUALIFIED_SYSTEM) return
        val name = (expression.selectorExpression as? KtCallExpression)?.calleeExpression?.text ?: return
        if (name !in TIME_CALLS) return
        val message = if (name == NANO_TIME_CALL) {
            "Replace System.nanoTime() with TimeSource.Monotonic from kotlin.time to measure an elapsed time."
        } else {
            "Replace System.$name() with Clock.System.now() from kotlin.time, injected as a Clock where the code " +
                "must be testable."
        }
        report(Finding(Entity.from(expression), message))
    }

    private companion object {
        const val SYSTEM = "System"
        const val QUALIFIED_SYSTEM = "java.lang.System"
        const val NANO_TIME_CALL = "nanoTime"

        @JvmField
        val TIME_CALLS = setOf("currentTimeMillis", NANO_TIME_CALL)
        const val DESCRIPTION = "Time is read from kotlin.time Clock and TimeSource, never from System time calls."
    }
}
