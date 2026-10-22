package ru.astrainteractive.klibs.rules.konsist.cli

import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import java.io.File

private val LINE_BREAKING = Regex("[\t\r\n]")

private fun Violation.toLine(): String {
    return listOf(
        rule,
        path,
        subject,
        message
    )
        .joinToString(separator = "\t") { field -> field.replace(LINE_BREAKING, " ") }
}

/**
 * Writes one violation per line as `rule<TAB>path<TAB>subject<TAB>message`; the file is empty when there is
 * no violation.
 */
internal fun writeReport(violations: List<Violation>, reportFile: File) {
    reportFile.parentFile?.mkdirs()
    reportFile.writeText(violations.joinToString(separator = "") { violation -> violation.toLine() + "\n" })
}
