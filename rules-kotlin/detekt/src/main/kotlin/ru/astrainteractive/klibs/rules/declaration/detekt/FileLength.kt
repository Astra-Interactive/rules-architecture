package ru.astrainteractive.klibs.rules.declaration.detekt

import dev.detekt.api.Config
import dev.detekt.api.Entity
import dev.detekt.api.Finding
import dev.detekt.api.Rule
import org.jetbrains.kotlin.psi.KtFile

/**
 * Reports a file that passes 400 lines: that is the point where a file is split, and the only one.
 */
internal class FileLength(config: Config) : Rule(config, DESCRIPTION) {

    override fun visitKtFile(file: KtFile) {
        super.visitKtFile(file)
        val lineCount = file.text.lines().size
        if (lineCount <= MAX_LINES) return
        val message = "${file.name} has $lineCount lines: split it, a file passes $MAX_LINES lines only by mistake."
        report(Finding(Entity.from(file), message))
    }

    private companion object {
        const val MAX_LINES = 400
        const val DESCRIPTION = "A file is split when it passes 400 lines."
    }
}
