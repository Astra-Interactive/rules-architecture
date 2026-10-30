package ru.astrainteractive.klibs.rules.time.detekt

import dev.detekt.api.Config
import dev.detekt.api.Entity
import dev.detekt.api.Finding
import dev.detekt.api.Rule
import org.jetbrains.kotlin.name.Name
import org.jetbrains.kotlin.psi.KtDotQualifiedExpression
import ru.astrainteractive.klibs.rules.detekt.util.isTestSource

/**
 * Reports `Clock.System` outside a `di` package: a class takes the `Clock` through its constructor, and the DI module
 * passes `Clock.System`. Test sources are not judged.
 */
internal class ClockSystemOutsideDi(config: Config) : Rule(config, DESCRIPTION) {

    override fun visitDotQualifiedExpression(expression: KtDotQualifiedExpression) {
        super.visitDotQualifiedExpression(expression)
        val receiver = expression.receiverExpression.text
        if (receiver != CLOCK && receiver != QUALIFIED_CLOCK) return
        if (expression.selectorExpression?.text != SYSTEM) return
        val file = expression.containingKtFile
        if (file.isTestSource()) return
        val segments = file.packageFqName.pathSegments().map(Name::asString)
        if (DI_SEGMENT in segments) return
        val message = "Take a Clock through the constructor instead of reading Clock.System here: the DI module " +
            "passes Clock.System."
        report(Finding(Entity.from(expression), message))
    }

    private companion object {
        const val CLOCK = "Clock"
        const val QUALIFIED_CLOCK = "kotlin.time.Clock"
        const val SYSTEM = "System"
        const val DI_SEGMENT = "di"
        const val DESCRIPTION = "A class takes the Clock through its constructor; only a DI module passes Clock.System."
    }
}
