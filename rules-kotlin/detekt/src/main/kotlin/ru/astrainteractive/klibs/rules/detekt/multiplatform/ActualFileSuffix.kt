package ru.astrainteractive.klibs.rules.detekt.multiplatform

import dev.detekt.api.Config
import dev.detekt.api.Entity
import dev.detekt.api.Finding
import dev.detekt.api.Rule
import org.jetbrains.kotlin.lexer.KtTokens
import org.jetbrains.kotlin.psi.KtFile
import org.jetbrains.kotlin.psi.KtModifierListOwner

/**
 * Reports a file holding an `actual` declaration whose name carries no platform suffix: `Clipboard.android.kt`,
 * not `Clipboard.kt`.
 */
internal class ActualFileSuffix(config: Config) : Rule(config, DESCRIPTION) {

    override fun visitKtFile(file: KtFile) {
        super.visitKtFile(file)
        if (SUFFIXED_NAME.matches(file.name)) return
        val actual = file.declarations
            .filterIsInstance<KtModifierListOwner>()
            .firstOrNull { declaration -> declaration.hasModifier(KtTokens.ACTUAL_KEYWORD) }
            ?: return
        val baseName = file.name.removeSuffix(KOTLIN_EXTENSION)
        val message = "Name the file of an actual declaration with its platform: $baseName.<platform>.kt, such " +
            "as $baseName.android.kt or $baseName.jvm.kt."
        report(Finding(Entity.from(actual), message))
    }

    private companion object {
        const val KOTLIN_EXTENSION = ".kt"

        @JvmField
        val SUFFIXED_NAME = Regex("""[^.]+\.[A-Za-z0-9]+\.kt""")
        const val DESCRIPTION = "A file of actual declarations carries its platform suffix."
    }
}
