package ru.astrainteractive.klibs.rules.konsist.cli.util

import ru.astrainteractive.klibs.rules.konsist.check.model.Violation

private val LINE_BREAKING = Regex("[\t\r\n]")

internal fun Violation.toLine(): String {
    return listOf(
        rule,
        path,
        subject,
        message
    )
        .joinToString(separator = "\t") { field -> field.replace(LINE_BREAKING, " ") }
}
