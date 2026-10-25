package ru.astrainteractive.klibs.rules.konsist.cli.konsist

import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import ru.astrainteractive.klibs.rules.konsist.cli.util.toLine
import java.io.File

/**
 * Writes one violation per line as `rule<TAB>path<TAB>subject<TAB>message`; the file is empty when there is
 * no violation.
 */
internal fun writeReport(violations: List<Violation>, reportFile: File) {
    reportFile.parentFile?.mkdirs()
    reportFile.writeText(violations.joinToString(separator = "") { violation -> violation.toLine() + "\n" })
}
